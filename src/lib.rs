use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde_json;
use std::fs;
use std::io::{self, Write};

use strava::activities::{list_activities, Lap};
use strava::streams::{get_streams, Streams};

pub static USER_AUTH: &str = "./auth/user.json";
pub static SECRETS: &str = "./auth/secrets.json";

pub fn is_setup() -> bool {
    // check if the user.json file exists for main to determine if we should run setup or not.
    if !fs::metadata(SECRETS).is_ok() {
        panic!("Error: Missing APP secrets");
    }
    fs::metadata(USER_AUTH).is_ok()
}

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

                        match fs::write(
                            format!("./activities/{}.json", activity.id),
                            serde_json::to_string_pretty(&activity).unwrap(),
                        ) {
                            Ok(_) => println!("Successful wrote activity {} to file", activity.id),
                            Err(_) => println!("Error writting activity {} to file", activity.id),
                        }
                        break;
                    }
                }
            } else {
                println!("There was insufficient data to get your splits.");
            }
        }
    } else {
        println!("No activities found!");
    }
}

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
