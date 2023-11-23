use crate::api::get_response;
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Streams {
    pub distance: DistanceStream,
    pub time: TimeStream,
    pub moving: MovingStream,
}

#[derive(Debug, Deserialize)]
pub struct DistanceStream {
    // each data point is how far the user has gone in meters
    // type: String,
    pub data: Vec<f32>,
    pub original_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct TimeStream {
    // each data point is how long the activity has been running in seconds
    pub data: Vec<i32>,
    pub original_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct MovingStream {
    // each data point is a bool on whether or not the user was moving
    pub data: Vec<bool>,
    pub original_size: usize,
}

pub fn get_streams(id: i64, keys: &str, access_token: &str) -> Option<Streams> {
    let path = format!("/activities/{}/streams", id);
    let params = format!("?keys={}&key_by_type=true", keys);

    if let Ok(response) = get_response(&path, &params, access_token) {
        if response.status == StatusCode::OK {
            let streams: Streams = serde_json::from_str(&response.body).unwrap();
            Some(streams)
        } else {
            None
        }
    } else {
        None
    }
}
