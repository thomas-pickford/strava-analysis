use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::sync::mpsc;

use crate::server;
use webbrowser;

#[derive(Debug, Serialize, Deserialize)]
struct Login {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Secrets {
    client_id: u32,
    client_secret: String,
}

type LoginResult = Result<Login, reqwest::Error>;

fn exchange_token(code: &str, id: u32, secret: &str) -> LoginResult {
    let mut body = HashMap::new();
    body.insert("client_id", format!("{}", id));
    body.insert("client_secret", String::from(secret));
    body.insert("code", String::from(code));
    body.insert("grant_type", String::from("authorization_code"));
    let mut res = reqwest::Client::new()
        .post("https://www.strava.com/oauth/token")
        .json(&body)
        .send()?
        .error_for_status()?;
    Ok(res.json()?)
}

fn auth_url(client_id: u32) -> String {
    let scopes = [
        "read_all",
        "profile:read_all",
        "activity:read_all",
        "activity:write",
    ]
    .join(",");

    let params = [
        format!("client_id={}", client_id),
        String::from("redirect_uri=http://localhost:8000"),
        String::from("response_type=code"),
        String::from("approval_prompt=auto"),
        format!("scope={}", scopes),
    ]
    .join("&");
    format!("https://www.strava.com/oauth/authorize?{}", params)
}

pub fn auth_new_user() {
    // Get API secrets from file
    let input = fs::read_to_string("auth\\secrets.json").expect("Unable to read client secrets");
    let secrets: Secrets = serde_json::from_str(&input).unwrap();

    // Direct user to auth url
    let auth_url = auth_url(secrets.client_id);
    if webbrowser::open(&auth_url).is_err() {
        // Try manually
        println!("Visit the following URL to authorize your app with Strava:");
        println!("{}\n", auth_url);
    }

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        server::start(tx);
    });

    // recv() is blocking, so the main thread will patiently
    // wait for data to be sent through the channel.
    // This way the server thread stays alive for as long as
    // it's needed.
    match rx.recv().unwrap() {
        Ok(auth_info) => {
            match exchange_token(&auth_info.code, secrets.client_id, &secrets.client_secret) {
                Ok(login) => {
                    if let Ok(results_str) = serde_json::to_string_pretty(&login) {
                        if let Err(err) = fs::write("auth\\user.json", results_str) {
                            eprintln!("Error saving auth tokens {}", err);
                            std::process::exit(1);
                        }
                        println!("Successfully authenticated <user>");
                    } else {
                        eprintln!("Error serializing auth tokens to JSON");
                        std::process::exit(1);
                    }
                }
                Err(error) => eprintln!("Error: {:#?}", error),
            }
        }
        Err(error) => {
            eprintln!("Error: {:#?}", error);
            // Let the async server send its response
            // before the main thread exits.
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

pub fn get_summary() {
    println!("Getting activity summaries for the day");
}

pub fn get_splits(interval: String) {
    println!("Getting {} splits from datastream", interval);
}

// fn is_setup() {
//     // check if the user.json file exists and return that information to the function calling.
//     // otherwise print an error and exit the program
// }
