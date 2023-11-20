use serde::Deserialize;

use crate::api::get_response;

#[derive(Debug, Deserialize)]
pub struct Activity {
    pub id: i64,
    pub name: String,
    pub distance: f32,
    pub moving_time: i32,
}

/// create activity

/// get activity

/// list activity comments

/// list activity kudoers

/// list activity laps

/// List Athlete Activities (getLoggedInAthleteActivities)
///
/// https://developers.strava.com/docs/reference/#api-Activities-getLoggedInAthleteActivities
///
pub fn list_activities(after: i64, before: i64, token: &str) -> Option<Vec<Activity>> {
    let path = "/athlete/activities";
    let params = format!("?before={}&after={}", before, after);

    if let Ok(response) = get_response(&path, &params, token) {
        let activities: Vec<Activity> = serde_json::from_str(&response).unwrap();
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
