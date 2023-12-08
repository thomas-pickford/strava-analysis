use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use std::fs;
use std::io::{self, Write};

use strava::activities::{Activity, Lap};
use strava::streams::Streams;

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
///
/// # Example
///
/// ```
/// use strava_analysis::is_setup;
///
/// if is_setup() {
///     println!("Program options: ");
/// } else {
///     println!("Authenticate the new user")
/// }
/// 
/// ```
pub fn is_setup() -> bool {
    // check if the user.json file exists for main to determine if we should run setup or not.
    if fs::metadata(SECRETS).is_err() {
        panic!("Error: Missing APP secrets");
    }
    fs::metadata(USER_AUTH).is_ok()
}

/// Retrieves the summary of activities within a specified time range and formats them based on the given lap size.
///
/// # Arguments
///
/// * `lap_size` - A string representing the lap_size of the activities (e.g., "1K" for kilometers, "MILE" for miles).
/// * `activity` - A reference to an `Activity` struct representing the activity details.
///
/// # Example
///
/// ```
/// use chrono::NaiveDateTime;
/// use strava_analysis::get_summary;
/// use strava::activities::Activity;
///
/// let lap_size = "1K".to_string();
/// let activity = Activity {
///     id: 123456789,
///     name: "Running".to_string(),
///     start_date_local: "2021-10-01T08:00:00Z".to_string(),
///     distance: 5000.0,
///     moving_time: 1800,
///     manual: false,
///     laps: None,
/// };
///
/// get_summary(&lap_size, &activity);
/// ```
pub fn get_summary(lap_size: &String, activity: &Activity) {
    println!("{}", activity.name);
    println!(
        "Date: {}",
        NaiveDateTime::parse_from_str(&activity.start_date_local, "%Y-%m-%dT%H:%M:%SZ")
            .expect("Bad date")
            .format("%m-%d-%Y")
    );
    if lap_size == "1K" {
        let distance = activity.distance / 1000.0;
        println!("Distance: {:.2}K", distance);
        println!(
            "Pace: {} min/k",
            format_time((activity.moving_time as f32 / distance).round() as i32)
        );
    } else if lap_size == "MILE" {
        let distance = activity.distance * 0.000621371;
        println!("Distance: {:.2}mi", distance);
        println!(
            "Pace: {} min/mi",
            format_time((activity.moving_time as f32 / distance).round() as i32)
        );
    }
    println!("Moving Time: {}\n", format_time(activity.moving_time));
}

/// Retrieves splits from the activity stream formatted by the distance lap_size.
///
/// # Arguments
///
/// * `lap_size` - The lap_size for the splits (e.g., "MILE", "1K").
/// * `streams` - The streams containing distance data.
///
/// # Returns
///
/// An optional vector of `Lap` structs representing the splits.
///
/// # Example
///
/// ```
/// use strava_analysis::get_splits;
/// use strava::streams::{Streams, DistanceStream, TimeStream, MovingStream};
/// use strava::activities::Lap;
///
/// let lap_size = String::from("MILE");
/// let streams = Streams {
///     distance: DistanceStream {
///         data: vec![10.0; 1], // Example distance data
///         original_size: 1,
///     },
///     time: TimeStream {
///         data: vec![100; 1], // Example time data
///         original_size: 1,
///     },
///     moving: MovingStream {
///         data: vec![true; 1], // Example moving data
///         original_size: 1,
///     },
/// };
///
/// let splits = get_splits(&lap_size, &streams);
/// if let Some(splits) = splits {
///     for lap in splits {
///         println!("Lap Name: {}", lap.name);
///         println!("Lap Distance: {}", lap.distance);
///         println!("Lap Moving Time: {}", lap.moving_time);
///     }
/// }
/// ```
pub fn get_splits(lap_size: &str, streams: &Streams) -> Option<Vec<Lap>> {
    let mut laps: Vec<Lap> = Vec::new();
    let mut format_lap_size = 0.0;
    match lap_size {
        "MILE" => format_lap_size = 1609.34,
        "1K" => format_lap_size = 1000.0,
        _ => println!("Shouldn't hit this"),
    }

    let mut lap_cnt = 1;
    let mut start: usize = 0;
    let mut cur: usize = 0;
    let mut distance: f32;
    let end = streams.distance.original_size;
    loop {
        if streams.distance.data[cur] / format_lap_size > lap_cnt as f32 {
            distance = streams.distance.data[cur] - streams.distance.data[start];
            let moving_time = calc_moving_time(start, cur, streams);
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
            if (streams.distance.data[end - 1] - streams.distance.data[start]) / format_lap_size >= 0.1 {
                distance = streams.distance.data[end - 1] - streams.distance.data[start];
                let moving_time = calc_moving_time(start, end - 1, streams);
                let lap = Lap {
                    name: format!("Lap {}", lap_cnt),
                    distance,
                    moving_time,
                };

                laps.push(lap);
            }
            return Some(laps);
        }
    }
}

/// Calculates the moving time between two points in the given `streams`.
///
/// The `start` and `end` points specify the range of data to consider in the `streams`.
/// The `streams` parameter should contain the relevant time and moving data.
///
/// The function iterates over the specified range and calculates the moving time by subtracting the stopped time from the elapsed time.
/// Stopped time is calculated by summing the time data points when the user was not moving.
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
/// 
/// # Example
/// 
/// ```
/// use strava_analysis::format_time;
/// 
/// let moving_time = 3600;
/// let time = format_time(moving_time);
/// 
/// assert_eq!(time, "1:00:00");
/// ```
pub fn format_time(moving_time: i32) -> String {
    let mut time = String::new();
    let mut min = moving_time / 60;
    let sec = moving_time % 60;
    if min >= 60 {
        let hour = min / 60;
        min %= 60;
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

/// Prompts the user to select a formatting lap_size.
///
/// The user is prompted to enter a lap size, either "mile" or "1k".
/// If the user enters a valid lap_size, it is returned as an `Option<String>`.
/// If the user cancels the request by entering "Q", `None` is returned.
/// If the user enters an unsupported distance, an error message is displayed and the prompt is repeated.
pub fn get_lap_size() -> Option<String> {
    println!("Pick formatting lap_size (mile, 1k):");
    loop {
        let mut lap_size = String::new();
        print!("lap_size> ");
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
        if before.is_ok() && after.is_ok() && (before.unwrap() >= after.unwrap()) {
            if let Some(lap_size) = get_lap_size() {
                println!();
                return Some((
                    lap_size,
                    NaiveDateTime::new(after.unwrap(), NaiveTime::from_hms_opt(0, 0, 0)?)
                        .timestamp(),
                    NaiveDateTime::new(before.unwrap(), NaiveTime::from_hms_opt(23, 59, 59)?)
                        .timestamp(),
                ));
            }
        } else {
            println!("Invalid date range entered. Please try again");
        }
    }
}

/// Calculates the summary of a week's activities and formats on the given distance lap_size.
///
/// # Arguments
///
/// * `lap_size` - A reference to a String representing the lap_size (e.g., "1K", "MILE").
/// * `activities` - A vector of Activity objects representing the activities for the week.
///
/// # Example
///
/// ```
/// use strava::activities::Activity;
///
/// let lap_size = String::from("1K");
/// let activities = vec![
///     Activity { id: 1, name: "run1".to_string(), distance: 500.0, moving_time: 1200, laps: None, start_date_local: "2021-10-01T08:00:00Z".to_string(), manual: false },
///     Activity { id: 2, name: "run2".to_string(), distance: 800.0, moving_time: 1800, laps: None, start_date_local: "2021-10-01T08:00:00Z".to_string(), manual: true },
///     Activity { id: 3, name: "run3".to_string(), distance: 1200.0, moving_time: 2400, laps: None, start_date_local: "2021-10-01T08:00:00Z".to_string(), manual: false },
/// ];
///
/// strava_analysis::get_week_summary(&lap_size, activities);
/// ```
pub fn get_week_summary(lap_size: &String, activities: Vec<Activity>) {
    let mut distance = 0.0;
    let mut moving_time = 0;
    for activity in activities {
        distance += activity.distance;
        moving_time += activity.moving_time;
    }
    println!("Week Overview");
    if lap_size == "1K" {
        distance /= 1000.0;
        println!("Distance: {:.2}K", distance);
        println!(
            "Pace: {} min/k",
            format_time((moving_time as f32 / distance).round() as i32)
        );
    } else if lap_size == "MILE" {
        distance *= 0.000621371;
        println!("Distance: {:.2}mi", distance);
        println!(
            "Pace: {} min/mi",
            format_time((moving_time as f32 / distance).round() as i32)
        );
    }
    println!("Moving Time: {}\n", format_time(moving_time));
}
