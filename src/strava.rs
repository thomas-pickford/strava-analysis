use serde::Deserialize;
use std::fs;
use chrono::{Local, Timelike};

use crate::auth::{get_token, USER_AUTH};

/// User interface functions
pub fn is_setup() -> bool {
    // check if the user.json file exists for main to determine if we should run setup or not.
    fs::metadata(USER_AUTH).is_ok()
}

pub fn greet_user() {
    let user = get_user_firstname();
        println!("Welcome back {}!", user);
}

#[derive(Debug, Deserialize)]
struct User {
    firstname: String,
}

pub fn get_user_firstname() -> String {
    let url = "https://www.strava.com/api/v3/athlete";
    let response = get_response(url);
    if response.is_ok() {
        let user: User = serde_json::from_str(&response.unwrap()).unwrap();
        user.firstname
    } else {
        String::new()
    }
}

#[derive(Debug, Deserialize)]
struct Activity {
    id: i64,
    name: String,
    distance: f32,
    moving_time: i32,
}

fn get_activities() -> Vec<Activity> {
    let today = Local::now();
    let before = today.with_hour(23).unwrap().with_minute(59).unwrap().with_second(59).unwrap().timestamp();
    let after = today.with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap().timestamp();

    let url = format!("https://www.strava.com/api/v3/athlete/activities?before={}&after={}", before, after);

    let response = get_response(&url);    
    if response.is_ok() {
        let activities: Vec<Activity> = serde_json::from_str(&response.unwrap()).unwrap();
        activities
    }
    else {
        Vec::<Activity>::new()
    }
}

pub fn get_summary() {
    let activities = get_activities();
    for activity in activities {
        // TODO: output distance in either mile/km
        // output total time and pace
        println!(
            "Name: {}, Distance: {}, Moving Time: {}",
            activity.name, activity.distance, format_time(activity.moving_time)
        );
    }
}

#[derive(Debug, Deserialize)]
struct Streams {
    distance: DistanceData,
    time: TimeData,
    moving: MovingData,
}

#[derive(Debug, Deserialize)]
struct DistanceData {
    // each data point is how far the user has gone in meters
    data: Vec<f32>,
    original_size: usize,
}

#[derive(Debug, Deserialize)]
struct TimeData {
    // each data point is how long the activity has been running in seconds
    data: Vec<i32>,
    original_size: usize,
}

#[derive(Debug, Deserialize)]
struct MovingData {
    // each data point is a bool on whether or not the user was moving
    data: Vec<bool>,
    original_size: usize,
}

// pub fn get_splits(interval: String) {
pub fn get_splits(interval: String) {
    let activities = get_activities();
    for activity in activities {
        let url = format!("https://www.strava.com/api/v3/activities/{}/streams?keys=distance,time,moving&key_by_type=true", activity.id);

        let response = get_response(&url);
        if response.is_ok() {
            let streams: Streams = serde_json::from_str(&response.unwrap()).unwrap();
            println!("{} splits for activity '{}'", interval, activity.name);
            let mut size = 0.0;
            match interval.as_str() {
                "MILE" => size = 1609.34,
                "1K" => size = 1000.0,
                _ => println!("Shouldn't hit this"),
            }

            if streams.distance.original_size != streams.time.original_size && streams.distance.original_size != streams.moving.original_size {
                // should never hit this
                println!("Corrupted data streams on activity {}. Unable to get splits", activity.id);
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
                    println!("Lap {} {} pace", cnt, format_pace(size, distance, moving_time));
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
}

fn format_time(moving_time: i32) -> String {
    let min = moving_time / 60;
    let sec = moving_time % 60;
    if sec < 10 {
        format!("{}:0{}", min, sec)
    } else {
        format!("{}:{}", min, sec)
    }
}

fn format_pace(expected_distance: f32, actual_distance: f32, moving_time: i32) -> String {
    let pace = (moving_time as f32 / (actual_distance / expected_distance)).round() as i32;
    format_time(pace)
}

type APIResponse = Result<String, reqwest::Error>;
fn get_response(url: &str) -> APIResponse {
    let token = get_token().access_token;

    let client = reqwest::blocking::Client::new();
    let response = client.get(url)
        .bearer_auth(&token)
        .send();

    match response {
        Ok(response) => Ok(String::from(&response.text().unwrap())),
        Err(err) => {
            println!("There was an error getting your activities, please try again.");
            Err(err)
        },
    }

}