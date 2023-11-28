use serde::{Deserialize, Serialize};

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
/// let after = 1698472800; // October 28, 2023 0:00:00 AM UTC
/// let before = 1699599599; // November 09 2023 23:59:59 PM UTC
/// let token = "your_access_token";
///
/// let activities = list_activities(after, before, token);
///
/// match activities {
///     Some(activities) => {
///         for activity in activities {
///             println!("Activity ID: {}", activity.id);
///             println!("Activity Name: {}", activity.name);
///             // ... other activity details
///         }
///     },
///     None => {
///         println!("No activities found within the specified time range.");
///     }
/// }
/// ```
pub fn list_activities(after: i64, before: i64, token: &str) -> Option<Vec<Activity>> {
    let path = "/athlete/activities";
    let params = format!("?before={}&after={}", before, after);

    if let Ok(response) = get(&path, &params, token) {
        let activities: Vec<Activity> = serde_json::from_str(&response.body).unwrap();
        if activities.len() > 0 {
            Some(activities)
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
