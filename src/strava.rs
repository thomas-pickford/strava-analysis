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
    
    let client = reqwest::blocking::Client::new();
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
        println!(
            "Name: {}, Distance: {}, Moving Time: {}",
            activity.name, activity.distance, activity.moving_time
        );
    }
}

#[derive(Debug, Deserialize)]
struct Streams {
    distance: DistanceData,
    time: TimeData,
}

#[derive(Debug, Deserialize)]
struct DistanceData {
    data: Vec<f32>,
    original_size: i32,
}

#[derive(Debug, Deserialize)]
struct TimeData {
    data: Vec<f32>,
    original_size: i32,
}

// pub fn get_splits(interval: String) {
pub fn get_splits() {
    let activities = get_activities();
    for activity in activities {
        let url = format!("https://www.strava.com/api/v3/activities/{}/streams?keys=distance,time&key_by_type=true", activity.id);

        let response = get_response(&url);
        if response.is_ok() {
            let streams: Streams = serde_json::from_str(&response.unwrap()).unwrap();
            println!("Activity {} has a stream size of {}", activity.id, streams.distance.original_size);
        } else {
            println!("There was an error getting your splits, please try again.");
        }
    }
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