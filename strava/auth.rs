use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::sync::mpsc;

use crate::api::{auth_url, exchange_token, refresh_token};
use crate::server;
use webbrowser;

// APP Secrets
// type TokenResult = Result<User, reqwest::Error>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppSecrets {
    pub client_id: u32,
    pub client_secret: String,
}

impl AppSecrets {
    /// Get the API app client secrets
    pub fn from_file(path: &str) -> AppSecrets {
        let input = fs::read_to_string(path).expect("Unable to read file");
        let secrets: AppSecrets = serde_json::from_str(&input).unwrap();
        secrets
    }
}

// User Auth Tokens
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    expires_at: i64,
    refresh_token: String,
}

impl AuthTokens {
    /// Requires path to user token and app secrets
    pub fn from_file(user: &str, app_secrets: &str) -> AuthTokens {
        // Get Access Token from file
        let secrets = AppSecrets::from_file(app_secrets);
        let input = fs::read_to_string(user).expect("Unable to read user token");
        let mut tokens: AuthTokens = serde_json::from_str(&input).unwrap();
        if tokens.expires_at < Local::now().timestamp() {
            match refresh_token(
                &tokens.refresh_token,
                secrets.client_id,
                secrets.client_secret,
            ) {
                Ok(refresh) => {
                    if let Ok(results_str) = serde_json::to_string_pretty(&refresh) {
                        // write to file
                        if let Err(err) = fs::write(user, &results_str) {
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
}

/// send the user to the auth url and wait for the response with the auth code
/// callers responsibility to store the tokens
pub fn auth_new_user(
    client_id: u32,
    client_secret: &str,
    scopes: &[&str],
) -> Result<String, String> {
    // Direct user to auth url
    let auth_url = auth_url(client_id, scopes);
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
            match exchange_token(&auth_info.code, client_id, client_secret) {
                Ok(response) => {
                    // filter out other return info
                    let tokens: AuthTokens = serde_json::from_str(&response).unwrap();
                    if let Ok(results_str) = serde_json::to_string_pretty(&tokens) {
                        Ok(results_str)
                    } else {
                        Err(String::from("Error serializing auth tokens to JSON"))
                    }
                }
                Err(_error) => Err(String::from("Error with exchanging token. Auth failed")),
            }
        }
        Err(error) => {
            // Let the async server send its response
            // before the main thread exits.
            std::thread::sleep(std::time::Duration::from_secs(1));
            Err(error)
        }
    }
}
