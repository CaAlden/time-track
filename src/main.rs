use std::io::{stdin, BufRead};

use chrono::{NaiveTime, Duration, Local};
use clap::Parser;
use atty::Stream;

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
fn parse_time(time_str: &str) -> NaiveTime {
    NaiveTime::parse_from_str(time_str, "%H:%M")
        .expect(&format!("Failed to parse time from {time_str}"))
}

fn all_at_once(is_terminal: bool) -> Vec<Duration> {
    if is_terminal {
        println!("Enter simple times one per line. Send an EOF character to sum all time spans");
    }

    let mut durations = vec![];
    let mut seen: Option<String> = None;
    for line in stdin().lock().lines() {
        let cleaned = line.expect("failed to read a line").trim().to_string();
        if let Some(previous) = &seen {
            let first = parse_time(previous);
            let mut second = parse_time(&cleaned);
            if second < first {
                second = second + Duration::hours(12);
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
    return durations;
}

fn live_spans() -> Vec<Duration> {
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
        println!("Ended with an open span from {unpaired}... Ignoring");
    }
    return durations;
}

fn get_prediction() -> NaiveTime {
    println!("Calculating an end time prediction...\nWhen did you start?\n");
    let mut start_time = String::new();
    let _ = stdin().read_line(&mut start_time);
    let start = parse_time(start_time.trim());
    println!("Started at {0}", start.format("%H:%M"));
    println!("How many minutes were you on break?\n");
    let mut break_time = String::new();
    let _ = stdin().read_line(&mut break_time);
    let break_duration = break_time.trim().parse::<i64>().unwrap();
    let work_time = Duration::hours(8) + Duration::minutes(break_duration);
    return start + work_time;
}

fn main() {
    let args = Args::parse();
    let is_terminal = atty::is(Stream::Stdin);

    if !is_terminal && !args.mode.supports_piped_input() {
        panic!("Cannot run {0} mode on piped input", args.mode);
    }

    if args.mode == Modes::Prediction {
        println!("Your work will end at {}", get_prediction());
        return;
    }

    let durations = match args.mode {
        Modes::TimeTable => all_at_once(is_terminal),
        Modes::Live => live_spans(),
        Modes::Prediction => panic!("Should have already returned before this"),
    };

    let total_minutes: i64  = durations.iter().map(|d| { d.num_minutes() }).sum();
    let minutes = total_minutes % 60;
    let hours = total_minutes / 60;

    println!("You have been working for {hours} hour(s) and {minutes} minute(s)");
}
