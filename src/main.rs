use std::io::{stdin, Read};
use chrono::{NaiveTime, Duration};

fn read_times() -> Vec<String> {
    println!("Write all times separated by newlines and finish with CTRL+D\n");
    let mut times = String::new();
    stdin().read_to_string(&mut times).expect("An error occurred reading a line");
    times.split("\n")
         .map(|s| { s.trim().to_string() })
         .collect()
}

fn parse_time(time_str: &str) -> NaiveTime {
    NaiveTime::parse_from_str(time_str, "%H:%M")
        .expect(&format!("Failed to parse time from {time_str}"))
}

fn main() {
    let times = read_times();
    let mut durations = vec![];
    for i in (0..times.len() - 1).step_by(2) {
        let first = parse_time(times.get(i).unwrap());
        let mut second = parse_time(times.get(i + 1).unwrap());
        if second < first {
            second = second + Duration::hours(12);
        }
        durations.push(second - first);
    }

    let total_minutes: i64  = durations.iter().map(|d| { d.num_minutes() }).sum();
    let minutes = total_minutes % 60;
    let hours = total_minutes / 60;

    println!("You have been working for {hours} hour(s) and {minutes} minute(s)");
}
