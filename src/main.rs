use std::io::{self, BufRead};
use chrono::{DateTime, Local};

use clap::Parser;
use anyhow::{anyhow, Result};
mod args;
mod time;

use args::Args;

fn main() -> Result<()> {
    let args = Args::parse();
    let stdin = io::stdin();
    let hours = args.hours.unwrap_or(
        if args.discount {
            0
        } else {
            8
        }
    );

    let target_minutes = if args.discount {
        (8 * 60) - ((hours * 60) + args.minutes)
    } else {
         hours * 60 + args.minutes
    };
    let (hrs, mins) = time::to_hrs_minutes(target_minutes);
    println!("Working for {}", time::show_time(hrs, mins));
    println!("Input times one per line. Send an EOF character to finish inputting...");
    let mut lines: Vec<String> = vec![];
    for line in stdin.lock().lines() {
        lines.push(line.expect("Issues when reading from stdin"));
    }

    // This is immeidately going to be turned back into a DateTime, which the doc says is fine
    #[allow(deprecated)]
    let midnight = Local::now().date().and_hms_opt(0, 0, 0).ok_or(anyhow!("Expected midnight to exist"))?;
    let times = time::from_stream(&midnight, lines.iter())?;

    let mut total_minutes: i64 = 0;
    let mut first: Option<DateTime<Local>> = None;
    for time in times {
        match first {
            None => {
                first = Some(time);
            },
            Some(prev) => {
                total_minutes += (time - prev).num_minutes();
                first = None
            }
        }
    }

    if let Some(remaining) = first {
        let now = Local::now();
        let now_str = now.format("%-I:%M %p");
        println!("Ended with unclosed span... assuming ending now: {}", now_str);
        total_minutes += (now - remaining).num_minutes();
    }

    println!("{}", time::get_charaterized_time_remaining(total_minutes, target_minutes));
    Ok(())
}
