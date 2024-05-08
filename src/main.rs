use std::io::{stdin, BufRead};

use chrono::{NaiveTime, Duration, Local, NaiveDateTime};
use clap::Parser;
use atty::Stream;
use anyhow::{anyhow, Result};

mod modes;
use modes::Modes;

/// A simple program to track time spans
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Specify how the program should behave
    #[clap(value_enum, default_value_t = Modes::default())]
    mode: Modes,
}

fn epoch() -> NaiveTime {
    NaiveTime::from_hms_opt(0, 0, 0).expect("0, 0, 0 to be valid arguments to from_hms_opt")
}

/// Given an arbitrary string, attempt to parse it as a naive time in the format HH:mm
fn parse_time(time_str: &str) -> Result<NaiveDateTime> {
    let time = NaiveTime::parse_from_str(time_str, "%H:%M")?;
    let naive_date = NaiveDateTime::from_timestamp_millis(
        time.signed_duration_since(epoch()).num_milliseconds(),
    ).ok_or(anyhow!("Should be able to make a datetime"))?;
    Ok(naive_date)
}

/// The default way of calculating time. Time values are given one per line and subsequent pairs of
/// lines are considered time spans. If an odd number of spans is given, then the final time value
/// is ignored.
fn all_at_once(is_terminal: bool) -> Result<Vec<Duration>> {
    if is_terminal {
        println!("Enter simple times one per line. Send an EOF character to sum all time spans");
    }

    let mut durations = vec![];
    let mut seen: Option<String> = None;
    for line in stdin().lock().lines() {
        let cleaned = line?.trim().to_string();
        if let Some(previous) = &seen {
            let first = parse_time(previous)?;
            let mut second = parse_time(&cleaned)?;

            // The following logic attempts to handle wrapping from AM -> PM and also from PM ->
            // AM but cannot handle multiple days in a single span
            if (second < first) && first.time().signed_duration_since(epoch()).num_hours() < 12 {
                // Assuming wrapped to PM
                // first = 8:00
                // second: 1:00
                second = second + Duration::hours(12);
            } else if second < first {
                // Wrapped to a new day because first was after noon.
                // first = 14:40
                // second = 1:30
                second = second + Duration::hours(24)
            }
            durations.push(second - first);
            seen = None;
        } else {
            seen = Some(cleaned.to_string());
        }
    }
    if let Some(unpaired) = seen {
        println!("Ended with an open span from {unpaired}... Ignoring");
    }
    return Ok(durations);
}

/// A "live" time tracker. Each time the user presses enter a new span is started or an existing
/// span is closed. The user may end the tracking at any time by sending an EOF command.
fn live_spans() -> Result<Vec<Duration>> {
    println!("Tracking Spans Live. Press ENTER to start a span\n");
    let mut durations = vec![];
    let mut seen: Option<NaiveTime> = None;
    for _line in stdin().lock().lines() {
        let mut now = Local::now().naive_local().time();
        if let Some(previous) = &seen {
            if now < *previous {
                now = now + Duration::hours(12);
            }
            let prev_time = previous.format("%H:%M");
            let now_time = now.format("%H:%M");
            println!("Closing span from {prev_time} - {now_time}");
            println!("Status: Away");
            durations.push(now - *previous);
            seen = None;
        } else {
            let now_time = now.format("%H:%M");
            println!("Starting span at {now_time}");
            println!("Status: Working");
            seen = Some(now);
        }
    }
    if let Some(unpaired) = seen {
        println!("Closing unpaired span now");
        let mut now = Local::now().naive_local().time();
        if now < unpaired {
            now = now + Duration::hours(12);
        }
        durations.push(now - unpaired);
    }
    return Ok(durations);
}

/// Predict when the given amount of work has completed based on when the user started working and
/// how long they expect to be on break total.
fn get_prediction(hours: i64) -> Result<NaiveTime> {
    println!("Calculating an end time prediction...\nWhen did you start?\n");
    let mut start_time = String::new();
    let _ = stdin().read_line(&mut start_time);
    let start = parse_time(start_time.trim())?;
    println!("Started at {0}", start.format("%H:%M"));
    println!("How many minutes were you on break?\n");
    let mut break_time = String::new();
    let _ = stdin().read_line(&mut break_time);
    let break_duration = break_time.trim().parse::<i64>()?;
    let work_time = Duration::hours(hours) + Duration::minutes(break_duration);
    return Ok(start.time() + work_time);
}

fn main() {
    let args = Args::parse();
    let is_terminal = atty::is(Stream::Stdin);

    if !is_terminal && !args.mode.supports_piped_input() {
        panic!("Cannot run {0} mode on piped input", args.mode);
    }

    if args.mode == Modes::Prediction {
        if let Ok(prediction) = get_prediction(8 /* hours */) {
            println!("Your work will end at {}", prediction);
        } else {
            println!("Something went wrong...");
        }
        return;
    }

    let maybe_durations = match args.mode {
        Modes::TimeTable => all_at_once(is_terminal),
        Modes::Live => live_spans(),
        Modes::Prediction => Err(anyhow::anyhow!("Should have already returned before this")),
    };

    match maybe_durations {
        Ok(durations) => {
            let total_minutes: i64  = durations.iter().map(|d| { d.num_minutes() }).sum();
            let minutes = total_minutes % 60;
            let hours = total_minutes / 60;
            let pluralized_hours = match hours {
                1 => "hour",
                _ => "hours",
            };
            let pluralized_minutes = match minutes {
                1 => "minute",
                _ => "minutes",
            };

            println!("You have been working for {hours} {pluralized_hours} and {minutes} {pluralized_minutes}");
        },
        Err(err) => eprintln!("{}\nExiting...", err),
    }
}
