use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde_json;
use std::fs;
use std::io::{self, Write};

use strava::activities::{list_activities, Lap};
use strava::streams::{get_streams, Streams};

pub static USER_AUTH: &str = "./auth/user.json";
pub static SECRETS: &str = "./auth/secrets.json";

/// Checks if the necessary setup has been completed.
///
/// This function checks if the `secrets.json` file exists in the specified path.
/// If the file does not exist, it panics with an error message indicating missing APP secrets.
/// It also checks if the `user.json` file exists.
///
/// # Returns
///
/// Returns `true` if the setup is complete, otherwise `false`.
pub fn is_setup() -> bool {
    // check if the user.json file exists for main to determine if we should run setup or not.
    if !fs::metadata(SECRETS).is_ok() {
        panic!("Error: Missing APP secrets");
    }
    fs::metadata(USER_AUTH).is_ok()
}

/// Retrieves the summary of activities within a specified time interval.
///
/// # Arguments
///
/// * `interval` - A string representing the interval of the activities (e.g., "1K" for kilometers, "MILE" for miles).
/// * `before` - An i64 representing the timestamp before which the activities should be retrieved.
/// * `after` - An i64 representing the timestamp after which the activities should be retrieved.
/// * `access_token` - A string slice representing the access token for authentication.
///
/// # Example
///
/// ```
/// let interval = "1K".to_string();
/// let before = 1635724800; // October 31, 2021 12:00:00 AM UTC
/// let after = 1633046400; // September 30, 2021 12:00:00 AM UTC
/// let access_token = "your_access_token";
///
/// get_summary(interval, before, after, access_token);
/// ```
pub fn get_summary(interval: String, before: i64, after: i64, access_token: &str) {
    if let Some(activities) = list_activities(after, before, access_token) {
        for activity in activities.iter().rev() {
            println!("{}", activity.name);
            println!(
                "Date: {}",
                NaiveDateTime::parse_from_str(&activity.start_date_local, "%Y-%m-%dT%H:%M:%SZ")
                    .expect("Bad date")
                    .format("%m-%d-%Y")
            );
            if interval == "1K" {
                let distance = activity.distance / 1000.0;
                println!("Distance: {:.2}K", distance);
                println!(
                    "Pace: {} min/k",
                    format_time((activity.moving_time as f32 / distance).round() as i32)
                );
            } else if interval == "MILE" {
                let distance = activity.distance * 0.000621371;
                println!("Distance: {:.2}mi", distance);
                println!(
                    "Pace: {} min/mi",
                    format_time((activity.moving_time as f32 / distance).round() as i32)
                );
            }
            println!("Moving Time: {}\n", format_time(activity.moving_time));
        }
    } else {
        println!("No activities found!");
    }
}

/// Retrieves splits for activities within a specified time interval.
///
/// # Arguments
///
/// * `interval` - The interval for the splits (e.g., "MILE", "1K").
/// * `before` - The timestamp for the end of the time interval.
/// * `after` - The timestamp for the start of the time interval.
/// * `access_token` - The access token for authentication.
///
/// # Example
///
/// ```
/// let interval = String::from("MILE");
/// let before = 1635724800; // October 31, 2021 12:00:00 AM UTC
/// let after = 1633046400; // September 30, 2021 12:00:00 AM UTC
/// let access_token = "your_access_token";
///
/// get_splits(interval, before, after, access_token);
/// ```
pub fn get_splits(interval: String, before: i64, after: i64, access_token: &str) {
    if let Some(activities) = list_activities(after, before, access_token) {
        for mut activity in activities {
            let mut laps: Vec<Lap> = Vec::new();

            let keys = ["distance", "time", "moving"].join(",");
            if let Some(streams) = get_streams(activity.id, &keys, access_token) {
                let mut lap_size = 0.0;
                match interval.as_str() {
                    "MILE" => lap_size = 1609.34,
                    "1K" => lap_size = 1000.0,
                    _ => println!("Shouldn't hit this"),
                }

                let mut lap_cnt = 1;
                let mut start: usize = 0;
                let mut cur: usize = 0;
                let mut distance: f32;
                let end = streams.distance.original_size;
                loop {
                    if streams.distance.data[cur] / lap_size > lap_cnt as f32 {
                        distance = streams.distance.data[cur] - streams.distance.data[start];
                        let moving_time = calc_moving_time(start, cur, &streams);
                        let lap = Lap {
                            name: format!("Lap {}", lap_cnt),
                            distance,
                            moving_time,
                        };

                        laps.push(lap);
                        start = cur;
                        lap_cnt += 1;
                    }
                    cur += 1;

                    // check for missed distance at the end less than the specified lap size
                    if cur == end {
                        if (streams.distance.data[end - 1] - streams.distance.data[start])
                            / lap_size
                            >= 0.1
                        {
                            distance =
                                streams.distance.data[end - 1] - streams.distance.data[start];
                            let moving_time = calc_moving_time(start, end - 1, &streams);
                            let lap = Lap {
                                name: format!("Lap {}", lap_cnt),
                                distance,
                                moving_time,
                            };

                            laps.push(lap);
                        }
                        activity.laps = Some(laps);
                        break;
                    }
                }
            } else {
                println!("Manual activity {} has no laps", activity.id);
            }
            let date =
                NaiveDateTime::parse_from_str(&activity.start_date_local, "%Y-%m-%dT%H:%M:%SZ")
                    .expect("Bad date")
                    .format("%m-%d");
            match fs::write(
                format!("./activities/{}-{}.json", date, activity.id),
                serde_json::to_string_pretty(&activity).unwrap(),
            ) {
                Ok(_) => println!("Successful wrote activity {} to file", activity.id),
                Err(_) => println!("Error writting activity {} to file", activity.id),
            }
        }
    } else {
        println!("No activities found!");
    }
}

/// Calculates the moving time between two indices in the given `streams`.
///
/// The `start` and `end` indices specify the range of data to consider in the `streams`.
/// The `streams` parameter should contain the relevant time and moving data.
///
/// The function iterates over the specified range and calculates the moving time by subtracting the stopped time from the elapsed time.
/// Stopped time is calculated by summing the time intervals when the user was not moving.
///
/// # Arguments
///
/// * `start` - The starting index of the range.
/// * `end` - The ending index of the range.
/// * `streams` - The streams containing time and moving data.
///
/// # Returns
///
/// The calculated moving time as an `i32` value.
pub fn calc_moving_time(start: usize, end: usize, streams: &Streams) -> i32 {
    let mut last_moving_time = 0;
    let mut stopped_time = 0;
    let elapsed_time = streams.time.data[end - 1] - streams.time.data[start];

    for i in start..=end - 1 {
        if streams.moving.data[i] {
            // user was moving
            last_moving_time = streams.time.data[i];
        } else {
            stopped_time += streams.time.data[i] - last_moving_time;
        }
    }

    // moving time
    elapsed_time - stopped_time
}

/// Formats the given moving time in seconds into a string representation of hours, minutes, and seconds.
///
/// # Arguments
///
/// * `moving_time` - The moving time in seconds.
///
/// # Returns
///
/// A string representation of the formatted time in the format "HH:MM:SS".
pub fn format_time(moving_time: i32) -> String {
    let mut time = String::new();
    let mut min = moving_time / 60;
    let sec = moving_time % 60;
    if min > 60 {
        let hour = min / 60;
        min = min % 60;
        if min < 10 {
            time.push_str(&format!("{}:0", hour));
        } else {
            time.push_str(&format!("{}:", hour));
        }
    }
    if sec < 10 {
        time.push_str(&format!("{}:0{}", min, sec));
    } else {
        time.push_str(&format!("{}:{}", min, sec));
    }

    time
}

/// Prompts the user to select a formatting interval for lap size.
///
/// The user is prompted to enter a lap size interval, either "mile" or "1k".
/// If the user enters a valid interval, it is returned as an `Option<String>`.
/// If the user cancels the request by entering "Q", `None` is returned.
/// If the user enters an unsupported distance, an error message is displayed and the prompt is repeated.
///
/// # Examples
///
/// ```
/// let lap_size = get_lap_size();
/// match lap_size {
///     Some(interval) => println!("Selected lap size interval: {}", interval),
///     None => println!("Request cancelled by user"),
/// }
/// ```
pub fn get_lap_size() -> Option<String> {
    println!("Pick formatting interval (mile, 1k):");
    loop {
        let mut lap_size = String::new();
        print!("interval> ");
        io::stdout().flush().expect("Failed to flush stdout");
        io::stdin()
            .read_line(&mut lap_size)
            .expect("failed to read input");
        let upper = lap_size.trim().to_ascii_uppercase();
        match upper.as_str() {
            "MILE" | "1K" => {
                println!();
                return Some(upper);
            }
            "Q" => {
                println!("Cancelled request");
                return None;
            }
            _ => println!("Unsupported distance. Please choose from the following (mile, 1k)"),
        }
    }
}

/// Prompts the user to enter a date range and returns it as a tuple.
/// The date range consists of a start date and an end date.
/// The user is prompted to enter the start date and end date in the format "MM/DD/YYYY".
/// If the user enters an invalid date range or chooses to quit by entering "q" or "Q",
/// the function returns `None`.
/// Otherwise, it returns `Some((lap_size, start_timestamp, end_timestamp))`,
/// where `lap_size` is obtained from the `get_lap_size()` function,
/// `start_timestamp` is the timestamp of the end of the start date,
/// and `end_timestamp` is the timestamp of the start of the end date.
pub fn get_date_range() -> Option<(String, i64, i64)> {
    println!("Example: \nstart> 11/08/2023\nend> 11/12/2023");
    loop {
        // get range start date
        let mut start = String::new();
        print!("start> ");
        io::stdout().flush().expect("Failed to flush stdout");
        io::stdin()
            .read_line(&mut start)
            .expect("failed to read input");
        start = start.trim().to_string();

        if start == "q" || start == "Q" {
            return None;
        }

        // get range end date
        let mut end = String::new();
        print!("end> ");
        io::stdout().flush().expect("Failed to flush stdout");
        io::stdin()
            .read_line(&mut end)
            .expect("failed to read input");
        end = end.trim().to_string();

        // verify valid dates and range
        let before = NaiveDate::parse_from_str(&end, "%m/%d/%Y");
        let after = NaiveDate::parse_from_str(&start, "%m/%d/%Y");
        if before.is_ok() && after.is_ok() && (before.unwrap() > after.unwrap()) {
            if let Some(lap_size) = get_lap_size() {
                println!();
                return Some((
                    lap_size,
                    NaiveDateTime::new(before.unwrap(), NaiveTime::from_hms_opt(23, 59, 59)?)
                        .timestamp(),
                    NaiveDateTime::new(after.unwrap(), NaiveTime::from_hms_opt(0, 0, 0)?)
                        .timestamp(),
                ));
            }
        } else {
            println!("Invalid date range entered. Please try again");
        }
    }
}

/// Calculates the summary of a week's activities based on the given parameters.
///
/// # Arguments
///
/// * `interval` - The interval for distance calculation. Valid values are "1K" and "MILE".
/// * `before` - The timestamp representing the end of the week.
/// * `after` - The timestamp representing the start of the week.
/// * `access_token` - The access token for authentication.
///
/// # Example
///
/// ```
/// let interval = "1K".to_string();
/// let before = 1635724800; // Timestamp for October 31, 2021
/// let after = 1635110400; // Timestamp for October 25, 2021
/// let access_token = "your_access_token";
///
/// get_week_summary(interval, before, after, access_token);
/// ```
pub fn get_week_summary(interval: String, before: i64, after: i64, access_token: &str) {
    if let Some(activities) = list_activities(after, before, access_token) {
        let mut distance = 0.0;
        let mut moving_time = 0;
        for activity in activities {
            distance += activity.distance;
            moving_time += activity.moving_time;
        }

        println!("Week Overview");
        if interval == "1K" {
            distance /= 1000.0;
            println!("Distance: {:.2}K", distance);
            println!(
                "Pace: {} min/k",
                format_time((moving_time as f32 / distance).round() as i32)
            );
        } else if interval == "MILE" {
            distance *= 0.000621371;
            println!("Distance: {:.2}mi", distance);
            println!(
                "Pace: {} min/mi",
                format_time((moving_time as f32 / distance).round() as i32)
            );
        }
        println!("Moving Time: {}\n", format_time(moving_time));
    }
}
