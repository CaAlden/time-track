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

fn adjust_last(first: &NaiveTime, second: NaiveTime) -> NaiveTime {
    // The following logic attempts to handle wrapping from AM -> PM and also from PM ->
    // AM but cannot handle multiple days in a single span
    if (second < *first) && first.signed_duration_since(epoch()).num_hours() < 12 {
        // Assuming wrapped to PM
        // first = 8:00
        // second: 1:00
        second + Duration::hours(12)
    } else if second < *first {
        // Wrapped to a new day because first was after noon.
        // first = 14:40
        // second = 1:30
        second + Duration::hours(24)
    } else {
        second
    }
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
            let second = adjust_last(&first.time(), parse_time(&cleaned)?.time());

            durations.push(second - first.time());
            seen = None;
        } else {
            seen = Some(cleaned.to_string());
        }
    }
    if let Some(unpaired) = seen {
        let now = Local::now().naive_local().time();
        let last = adjust_last(&now, parse_time(&unpaired)?.time());
        println!("Ended with an open span from {unpaired}... assuming now: {}", now.format("%H:%M"));
        durations.push(now - last);
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
            now = adjust_last(previous, now);
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
        now = adjust_last(&unpaired, now);
        durations.push(now - unpaired);
    }
    return Ok(durations);
}

fn to_hrs_minutes(total_minutes: i64) -> (i64, i64) {
    let minutes = total_minutes % 60;
    let hours = total_minutes / 60;
    (hours, minutes)
}

fn show_time(hours: i64, minutes: i64) -> String {
    let pluralized_hours = match hours {
        1 => "1 hour".to_string(),
        _ => format!("{hours} hours"),
    };
    let pluralized_minutes = match minutes {
        1 => "1 minute".to_string(),
        _ => format!("{minutes} minutes"),
    };

    if hours == 0 {
        return pluralized_minutes.to_string();
    }

    if minutes == 0 {
        return pluralized_hours.to_string();
    }

    return format!("{pluralized_hours} and {pluralized_minutes}");
}

fn get_charaterized_time_remaining(total_minutes: i64, target_minutes: i64) -> String {
    if total_minutes == target_minutes {
        return "Exactly done".to_string();
    }

    if total_minutes > target_minutes {
        let diff = total_minutes - target_minutes;
        let (hours, minutes) = to_hrs_minutes(diff);
        return format!("You have overworked {}", show_time(hours, minutes))
    } else {
        let diff = target_minutes - total_minutes;
        let (hours, minutes) = to_hrs_minutes(diff);
        let end_at = (Local::now() + Duration::minutes(diff)).time();
        let end_str = end_at.format("%-I:%M %p");
        return format!("You have {} remaining (end at {} starting now)", show_time(hours, minutes), end_str)
    }
}

fn main() {
    let args = Args::parse();
    let is_terminal = atty::is(Stream::Stdin);

    if !is_terminal && !args.mode.supports_piped_input() {
        panic!("Cannot run {0} mode on piped input", args.mode);
    }

    let maybe_durations = match args.mode {
        Modes::TimeTable => all_at_once(is_terminal),
        Modes::Live => live_spans(),
    };

    match maybe_durations {
        Ok(durations) => {
            let total_minutes: i64  = durations.iter().map(|d| { d.num_minutes() }).sum();
            let (hours, minutes) = to_hrs_minutes(total_minutes);

            println!("-----------------");
            println!("You have been working for {}", show_time(hours, minutes));
            println!("{}", get_charaterized_time_remaining(total_minutes, 8 * 60))
        },
        Err(err) => eprintln!("{}\nExiting...", err),
    }
}
