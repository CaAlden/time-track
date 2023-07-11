use std::io::{stdin, BufRead};

use chrono::{NaiveTime, Duration, Utc};
use clap::Parser;
use atty::Stream;

/// A simple program to track time spans
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// When passed the script will collect time spans live. Close a span by pressing ENTER and
    /// finish the calulation by passing in an EOF character
    #[arg(short, long, default_value_t = false)]
    live: bool,
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
        let mut now = Utc::now().naive_local().time();
        if let Some(previous) = &seen {
            if now < *previous {
                now = now + Duration::hours(12);
            }
            println!("Closing span from {previous} - {now}");
            println!("Status: Away");
            durations.push(now - *previous);
            seen = None;
        } else {
            println!("Starting span at {now}");
            println!("Status: Working");
            seen = Some(now);
        }
    }
    if let Some(unpaired) = seen {
        println!("Ended with an open span from {unpaired}... Ignoring");
    }
    return durations;
}

fn main() {
    let args = Args::parse();
    let is_terminal = atty::is(Stream::Stdin);

    if args.live && !is_terminal {
        panic!("Cannot run live mode on piped input");
    }

    let durations = if args.live {
        live_spans()
    } else {
        all_at_once(is_terminal)
    };

    let total_minutes: i64  = durations.iter().map(|d| { d.num_minutes() }).sum();
    let minutes = total_minutes % 60;
    let hours = total_minutes / 60;

    println!("You have been working for {hours} hour(s) and {minutes} minute(s)");
}
