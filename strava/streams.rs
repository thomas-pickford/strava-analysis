/// This module contains the implementation of activity streams in the Strava API.
///
/// The `Streams` struct represents the streams of an activity, including distance, time, and moving status.
/// The `DistanceStream` struct represents the distance stream of an activity, with each data point indicating the distance the user has gone in meters.
/// The `TimeStream` struct represents the time stream of an activity, with each data point indicating the duration of the activity in seconds.
/// The `MovingStream` struct represents the moving stream of an activity, with each data point indicating whether the user was moving or not.
///
/// The `get_streams` function retrieves the activity streams for a given activity ID, specified keys, and access token.
/// It returns an `Option` containing the streams if the request is successful, or `None` otherwise.
use crate::api::get;
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
    pub data: Vec<f32>,
    pub original_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct TimeStream {
    pub data: Vec<i32>,
    pub original_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct MovingStream {
    pub data: Vec<bool>,
    pub original_size: usize,
}

/// Get Activity Streams ([getActivityStreams](https://developers.strava.com/docs/reference/#api-Streams-getActivityStreams))
///
/// # Arguments
///
/// * `id` - The ID of the activity.
/// * `keys` - The keys of the streams to retrieve.
/// * `access_token` - The access token for authentication.
///
/// # Returns
///
/// An `Option` containing the streams if the request is successful, or `None` otherwise.
///
/// # Example
///
/// ```
/// use strava::streams::get_streams;
/// 
/// let id = 12345;
/// let keys = "distance,time,moving";
/// let access_token = "your_access_token";
///
/// if let Some(streams) = get_streams(id, keys, access_token) {
///     // Process the streams
///     println!("{:?}", streams);
/// } else {
///     println!("Failed to retrieve activity streams");
/// }
/// ```
pub fn get_streams(id: i64, keys: &str, access_token: &str) -> Option<Streams> {
    let path = format!("/activities/{}/streams", id);
    let params = format!("?keys={}&key_by_type=true", keys);

    if let Ok(response) = get(&path, &params, access_token) {
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
