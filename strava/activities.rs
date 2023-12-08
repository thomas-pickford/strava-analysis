use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::api::get;

use serde_json;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Activity {
    pub id: i64,
    pub name: String,
    pub distance: f32,
    pub moving_time: i32,
    pub manual: bool,
    pub start_date_local: String,
    pub laps: Option<Vec<Lap>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lap {
    pub name: String,
    pub distance: f32,
    pub moving_time: i32,
}

impl Activity {
    /// Saves the activity data to a JSON file.
    ///
    /// The activity data is saved in a file with the format "{date}-{id}.json",
    /// where {date} is the formatted start date of the activity in the format "MM-DD",
    /// and {id} is the unique identifier of the activity.
    ///
    /// # Examples
    ///
    /// ```
    /// use strava::activities::Activity;
    /// 
    /// let activity = Activity {
    ///     id: 123,
    ///     name: "Running".to_string(),
    ///     start_date_local: "2023-10-15T08:30:00Z".to_string(),
    ///     distance: 5000.0,
    ///     moving_time: 1800,
    ///     manual: false,
    ///     laps: None,
    /// };
    ///
    /// activity.save_to_json();
    /// ```
    pub fn save_to_json(&self) {
        let date = NaiveDateTime::parse_from_str(&self.start_date_local, "%Y-%m-%dT%H:%M:%SZ")
            .expect("Bad date")
            .format("%m-%d");
        match fs::write(
            format!("./activities/{}-{}.json", date, self.id),
            serde_json::to_string_pretty(self).unwrap(),
        ) {
            Ok(_) => println!("Successful wrote activity {} to file", self.id),
            Err(_) => println!("Error writting activity {} to file", self.id),
        }
    }
}

// create activity

// get activity

// list activity comments

// list activity kudoers

// list activity laps

/// List Athlete Activities ([getLoggedInAthleteActivities](https://developers.strava.com/docs/reference/#api-Activities-getLoggedInAthleteActivities))
///
/// # Arguments
///
/// * `after` - The starting date and time (in Unix timestamp format) for the activity search range.
/// * `before` - The ending date and time (in Unix timestamp format) for the activity search range.
/// * `token` - The access token for the authenticated user.
///
/// # Returns
///
/// Returns an `Option` containing a vector of `Activity` objects if the request is successful and there are activities found within the specified time range. Returns `None` otherwise.
///
/// # Example
///
/// ```
/// use strava::activities::list_activities;
/// 
/// let after = 1698472800; // October 28, 2023 0:00:00 AM UTC
/// let before = 1699599599; // November 09 2023 23:59:59 PM UTC
/// let token = "your_access_token";
///
/// if let Some(activities) = list_activities(after, before, token) {
///     for activity in activities {
///         println!("Activity ID: {}", activity.id);
///         println!("Activity Name: {}", activity.name);
///         // ... other activity details
///     }
/// } else {
///     println!("No activities found within the specified time range.");
/// }
/// ```
pub fn list_activities(after: i64, before: i64, token: &str) -> Option<Vec<Activity>> {
    let path = "/athlete/activities";
    let params = format!("?before={}&after={}", before, after);

    if let Ok(response) = get(path, &params, token) {
        let activities: Result<Vec<Activity>, _> = serde_json::from_str(&response.body);
        if let Ok(activities) = activities {
            if !activities.is_empty() {
                Some(activities)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        println!("None");
        None
    }
}

// get activity zones

// update activity
