use chrono::{
    // Duration,
    Local,
    Timelike,
};
use std::fs;
use std::io::{self, Write};

use strava::activities::list_activities;
use strava::auth::AuthTokens;
use strava::streams::get_streams;

static USER_AUTH: &str = "auth\\user.json";
static SECRETS: &str = "auth\\secrets.json";

fn is_setup() -> bool {
    // check if the user.json file exists for main to determine if we should run setup or not.
    if !fs::metadata(SECRETS).is_ok() {
        println!("Error: Missing APP secrets");
        return false;
    }
    fs::metadata(USER_AUTH).is_ok()
}

fn get_summary(interval: String, access_token: &str) {
    // let today = Local::now() - Duration::days(1);
    let today = Local::now();
    let before = today
        .with_hour(23)
        .unwrap()
        .with_minute(59)
        .unwrap()
        .with_second(59)
        .unwrap()
        .timestamp();
    let after = today
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .timestamp();

    if let Some(activities) = list_activities(after, before, access_token) {
        for activity in activities {
            println!("{}", activity.name);
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
            println!("Moving Time: {}", format_time(activity.moving_time));
        }
    } else {
        println!("No activities found today");
    }
}

pub fn get_splits(interval: String, access_token: &str) {
    let today = Local::now();
    let before = today
        .with_hour(23)
        .unwrap()
        .with_minute(59)
        .unwrap()
        .with_second(59)
        .unwrap()
        .timestamp();
    let after = today
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .timestamp();

    if let Some(activities) = list_activities(after, before, access_token) {
        for activity in activities {
            let keys = ["distance", "time", "moving"].join(",");
            if let Some(streams) = get_streams(activity.id, &keys, access_token) {
                println!("{} splits for activity '{}'", interval, activity.name);
                let mut size = 0.0;
                match interval.as_str() {
                    "MILE" => size = 1609.34,
                    "1K" => size = 1000.0,
                    _ => println!("Shouldn't hit this"),
                }

                if streams.distance.original_size != streams.time.original_size
                    && streams.distance.original_size != streams.moving.original_size
                {
                    // should never hit this
                    println!(
                        "Corrupted data streams on activity {}. Unable to get splits",
                        activity.id
                    );
                    continue;
                }

                let mut cnt = 1;
                let mut start: usize = 0;
                let mut cur: usize = 0;
                let mut distance: f32;
                let end = streams.distance.original_size;
                // let mut stopped_sum = 0;
                while cur < end {
                    if streams.distance.data[cur] / size > cnt as f32 {
                        distance = streams.distance.data[cur] - streams.distance.data[start];

                        let mut last_moving_time = 0;
                        let mut stopped_time = 0;
                        let elapsed_time = streams.time.data[cur] - streams.time.data[start];

                        for i in start..=cur {
                            if streams.moving.data[i] {
                                // user was moving
                                last_moving_time = streams.time.data[i];
                            } else {
                                stopped_time += streams.time.data[i] - last_moving_time;
                            }
                        }
                        // stopped_sum += stopped_time;

                        let moving_time = elapsed_time - stopped_time;
                        println!(
                            "Lap {} {} pace",
                            cnt,
                            calc_pace(size, distance, moving_time)
                        );
                        start = cur;
                        cnt += 1;
                    }
                    cur += 1;
                }
                // println!("Stopped sum: {}", stopped_sum);

                // println!("Activity {} has a stream size of {}", activity.id, streams.distance.original_size);
            } else {
                println!("There was an error getting your splits, please try again.");
            }
        }
    } else {
        println!("No activities found today");
    }
}

fn format_time(moving_time: i32) -> String {
    let mut time = String::new();
    let mut min = moving_time / 60;
    let sec = moving_time % 60;
    if min > 60 {
        let hour = min / 60;
        min = min % 60;
        time.push_str(&format!("{}:", hour));
    }
    if sec < 10 {
        time.push_str(&format!("{}:0{}", min, sec));
    } else {
        time.push_str(&format!("{}:{}", min, sec));
    }

    time
}

fn calc_pace(expected_distance: f32, actual_distance: f32, moving_time: i32) -> String {
    let pace = (moving_time as f32 / (actual_distance / expected_distance)).round() as i32;
    format_time(pace)
}

fn main() {
    let scopes = [
        "read_all",
        "profile:read_all",
        "activity:read_all",
        "activity:write",
    ];

    if is_setup() {
        // strava::strava::greet_user();
        println!("Welcome back!");
    } else {
        let secrets = strava::auth::AppSecrets::from_file(SECRETS);
        if let Ok(auth_resp) =
            strava::auth::auth_new_user(secrets.client_id, &secrets.client_secret, &scopes)
        {
            println!("{}", auth_resp);
            match fs::write(USER_AUTH, auth_resp) {
                Ok(_success) => println!("Successfully authenticated new user"),
                Err(error) => panic!("Error: Unable to write response to file:\n{}", error),
            }
        } else {
            panic!("Error: Unable to authenticating user. Please try again");
        }
    }

    let user = AuthTokens::from_file(USER_AUTH, SECRETS);

    println!("\nHow can I help you today?");
    println!("1. Get an overview of your running activities today");
    println!("2. Get the splits from your running activities today");
    println!("q. Quit");

    loop {
        let mut input = String::new();
        print!("> ");
        io::stdout().flush().expect("Failed to flush stdout");
        io::stdin()
            .read_line(&mut input)
            .expect("failed to read input");

        let trimmed_input = input.trim();

        match trimmed_input {
            "1" | "2" => {
                println!("Pick formatting interval (mile, 1k):");
                loop {
                    let mut interval_size = String::new();
                    print!("interval> ");
                    io::stdout().flush().expect("Failed to flush stdout");
                    io::stdin()
                        .read_line(&mut interval_size)
                        .expect("failed to read input");
                    let upper = interval_size.trim().to_ascii_uppercase();
                    match upper.as_str() {
                        "MILE" | "1K" => {
                            match trimmed_input {
                                "1" => get_summary(upper, &user.access_token),
                                "2" => get_splits(upper, &user.access_token),
                                _ => todo!(),
                            }
                            break;
                        }
                        "Q" => {
                            println!("Cancelled request");
                            break;
                        }
                        _ => println!(
                            "Unsupported distance. Please choose from the following (mile, 1k)"
                        ),
                    }
                }
            }
            "q" => {
                println!("Quitting the app. Goodbye!");
                break;
            }
            _ => println!("Invalid option. Try again."),
        }
    }
}
