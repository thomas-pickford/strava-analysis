use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::sync::mpsc;
use chrono::Local;

use crate::strava::get_user_firstname;
use crate::server;
use webbrowser;

pub static USER_AUTH: &str = "auth\\user.json";
static SECRETS: &str = "auth\\secrets.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    expires_at: i64,
    refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Secrets {
    client_id: u32,
    client_secret: String,
}

/// Get the API app client secrets
fn get_secrets() -> Secrets {
    let input = fs::read_to_string(SECRETS).expect("Unable to read client secrets");
    let secrets: Secrets = serde_json::from_str(&input).unwrap();
    secrets
}

/// API token helper functions

type TokenResult = Result<Token, reqwest::Error>;
// get the tokens from file, refresh and update if needed
pub fn get_token() -> Token {
    // Get API secrets from file
    let input = fs::read_to_string(USER_AUTH).expect("Unable to read client secrets");
    let mut tokens: Token = serde_json::from_str(&input).unwrap();
    if tokens.expires_at < Local::now().timestamp() {
        match refresh_token(&tokens.refresh_token) {
            Ok(refresh) => {
                if let Ok(results_str) = serde_json::to_string_pretty(&refresh) {
                    // write to file
                    if let Err(err) = fs::write(USER_AUTH, &results_str) {
                        eprintln!("Error saving auth tokens {}", err);
                        std::process::exit(1);
                    }
                    tokens = serde_json::from_str(&results_str).unwrap();
                } else {
                    eprintln!("Error serializing auth tokens to JSON");
                    std::process::exit(1);
                }
            }
            Err(error) => eprintln!("Error: {:#?}", error),
        }
    } 
    tokens

}

// refresh the user auth token
fn refresh_token(refresh: &str) -> TokenResult {
    // not sure if this works yet as of 11/13, haven't had the token expire...
    let secrets = get_secrets();
    let mut body = HashMap::new();
    body.insert("client_id", format!("{}", secrets.client_id));
    body.insert("client_secret", String::from(secrets.client_secret));
    body.insert("grant_type", String::from("refresh_token"));
    body.insert("refresh_token", String::from(refresh));
    let response = reqwest::blocking::Client::new()
        .post("https://www.strava.com/oauth/token")
        .json(&body)
        .send()?
        .error_for_status()?;

    Ok(response.json()?)
}


/// First time user authentication 
/// get the auth and refresh tokens
fn exchange_token(code: &str, id: u32, secret: &str) -> TokenResult {
    let mut body = HashMap::new();
    body.insert("client_id", format!("{}", id));
    body.insert("client_secret", String::from(secret));
    body.insert("code", String::from(code));
    body.insert("grant_type", String::from("authorization_code"));
    let response = reqwest::blocking::Client::new()
        .post("https://www.strava.com/oauth/token")
        .json(&body)
        .send()?
        .error_for_status()?;
    Ok(response.json()?)
}

/// create the auth url
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

/// send the user to the auth url and wait for the response with the auth code
pub fn auth_new_user() {
    // Get API secrets from file
    let secrets = get_secrets();

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
            // get the auth token
            match exchange_token(&auth_info.code, secrets.client_id, &secrets.client_secret) {
                Ok(tokens) => {
                    if let Ok(results_str) = serde_json::to_string_pretty(&tokens) {
                        // write to file
                        if let Err(err) = fs::write(USER_AUTH, results_str) {
                            eprintln!("Error saving auth tokens {}", err);
                            std::process::exit(1);
                        }
                        // get users name
                        let user = get_user_firstname();
                        println!("Welcome, {}! You are officially setup to start using the other features.", user);

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