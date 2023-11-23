use chrono::{Datelike, Duration, Local, NaiveDateTime, NaiveTime, Timelike};
use std::fs;
use std::io::{self, Write};

use strava::auth::AuthTokens;
use strava_analysis::*;

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
    println!("1. Get an overview of todays activities");
    println!("2. Get the splits from todays activities");
    println!("3. Get the splits from activities in a date range (mm/dd/yyyy)");
    println!("4. Get an overview of activities in a date range (mm/dd/yyyy)");
    println!("5. Get an overview of this weeks totals");
    println!("q. Quit");

    loop {
        let mut input = String::new();
        print!("> ");
        io::stdout().flush().expect("Failed to flush stdout");
        io::stdin()
            .read_line(&mut input)
            .expect("failed to read input");

        let trimmed_input = input.trim();

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

        match trimmed_input {
            "1" => {
                if let Some(lap_size) = get_lap_size() {
                    get_summary(lap_size, before, after, &user.access_token);
                }
            }
            "2" => {
                if let Some(lap_size) = get_lap_size() {
                    get_splits(lap_size, before, after, &user.access_token);
                }
            }
            "3" => {
                if let Some(params) = get_date_range() {
                    get_splits(params.0, params.1, params.2, &user.access_token);
                }
            }
            "4" => {
                if let Some(params) = get_date_range() {
                    get_summary(params.0, params.1, params.2, &user.access_token);
                }
            }
            "5" => {
                if let Some(lap_size) = get_lap_size() {
                    let today = Local::now().date_naive();
                    let weekday = today.weekday().num_days_from_sunday();
                    let week_start = NaiveDateTime::new(
                        today - Duration::days((weekday - 1) as i64),
                        NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                    );
                    let week_end = NaiveDateTime::new(
                        today + Duration::days((7 - weekday) as i64),
                        NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
                    );

                    get_week_summary(
                        lap_size,
                        week_end.timestamp(),
                        week_start.timestamp(),
                        &user.access_token,
                    );
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
