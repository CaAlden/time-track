use chrono::{Duration, Local, DateTime};
use anyhow::{Result, anyhow};
use std::iter::Iterator;
use regex::Regex;

fn parse_datetime(reference_date: &DateTime<Local>, s: &str) -> Result<DateTime<Local>> {
    let re = Regex::new(r"^\s*([12]?\d):([012345]\d)\s*$")?;
    let (_, [hrs, mins]) = re.captures(s).ok_or(anyhow!(format!("Failed to parse \"{s}\" as a time")))?.extract();
    let mins: u32 = mins.parse()?;
    let mut hrs: u32 = hrs.parse()?;
    let mut date = *reference_date;

    if hrs > 23 {
        let num_days = hrs / 24;
        date = *reference_date + Duration::days(num_days.into());
        hrs = hrs % 24;
    }
    #[allow(deprecated)]
    Ok(date.date().and_hms_opt(hrs, mins, 0).ok_or(anyhow!(format!("{}:{:0>2} not a real time", hrs, mins)))?)
}

pub fn from_stream<'a>(reference_date: &DateTime<Local>, stream: impl Iterator<Item = &'a String>) -> Result<Vec<DateTime<Local>>> {
    let mut durations: Vec<DateTime<Local>> = vec![];
    for val in stream {
        durations.push(parse_datetime(reference_date, val)?);
    }
    return Ok(durations);
}

pub fn show_time(hours: i64, minutes: i64) -> String {
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

pub fn to_hrs_minutes(total_minutes: i64) -> (i64, i64) {
    let minutes = total_minutes % 60;
    let hours = total_minutes / 60;
    (hours, minutes)
}

pub fn get_charaterized_time_remaining(
    total_minutes: i64,
    target_minutes: i64,
    ended_at: DateTime<Local>,
) -> String {
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
        let end_at = (ended_at + Duration::minutes(diff)).time();
        let end_str = end_at.format("%-I:%M %p");
        return if ended_at > Local::now() {
          format!(
              "You have {} remaining (end at {} starting from {})",
              show_time(hours, minutes),
              end_str,
              ended_at.format("%-I:%M %p"),
          )
        } else {
          format!("You have {} remaining (end at {} starting now)", show_time(hours, minutes), end_str)
        }
    }
}
