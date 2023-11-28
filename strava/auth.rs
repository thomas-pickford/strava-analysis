use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::sync::mpsc;

use crate::api::{auth_url, exchange_token, refresh_token};
use crate::server;
use webbrowser;

/// Represents the secrets required for the Strava API app.
#[derive(Debug, Serialize, Deserialize)]
pub struct AppSecrets {
    pub client_id: u32,
    pub client_secret: String,
}

impl AppSecrets {
    /// Get the API app client secrets from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file containing the app secrets.
    ///
    /// # Returns
    ///
    /// The `AppSecrets` struct containing the client ID and client secret.
    pub fn from_file(path: &str) -> AppSecrets {
        let input = fs::read_to_string(path).expect("Unable to read file");
        let secrets: AppSecrets = serde_json::from_str(&input).unwrap();
        secrets
    }
}

/// Represents the authentication tokens for a user.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    expires_at: i64,
    refresh_token: String,
}

impl AuthTokens {
    /// Get the authentication tokens for a user from a file.
    ///
    /// # Arguments
    ///
    /// * `user` - The path to the file containing the user token.
    /// * `app_secrets` - The path to the file containing the app secrets.
    ///
    /// # Returns
    ///
    /// The `AuthTokens` struct containing the access token, expiration timestamp, and refresh token.
    pub fn from_file(user: &str, app_secrets: &str) -> AuthTokens {
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
                    tokens = serde_json::from_str(&refresh.body).unwrap();
                    if let Err(err) =
                        fs::write(user, serde_json::to_string_pretty(&tokens).unwrap())
                    {
                        eprintln!("Error saving auth tokens {}", err);
                        std::process::exit(1);
                    }
                }
                Err(error) => eprintln!("Error: {:#?}", error),
            }
        }
        tokens
    }
}

/// Sends the user to the authentication URL and waits for the response with the authorization code.
/// It is the caller's responsibility to store the tokens.
///
/// # Arguments
///
/// * `client_id` - The client ID of the app.
/// * `client_secret` - The client secret of the app.
/// * `scopes` - The scopes required for the authorization.
///
/// # Returns
///
/// The authorization code as a `Result` containing either the code as a `String` or an error message as a `String`.
pub fn auth_new_user(
    client_id: u32,
    client_secret: &str,
    scopes: &[&str],
) -> Result<String, String> {
    let auth_url = auth_url(client_id, scopes);
    if webbrowser::open(&auth_url).is_err() {
        println!("Visit the following URL to authorize your app with Strava:");
        println!("{}\n", auth_url);
    }

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        server::start(tx);
    });

    match rx.recv().unwrap() {
        Ok(auth_info) => match exchange_token(&auth_info.code, client_id, client_secret) {
            Ok(response) => {
                let tokens: AuthTokens = serde_json::from_str(&response.body).unwrap();
                if let Ok(results_str) = serde_json::to_string_pretty(&tokens) {
                    Ok(results_str)
                } else {
                    Err(String::from("Error serializing auth tokens to JSON"))
                }
            }
            Err(_error) => Err(String::from("Error with exchanging token. Auth failed")),
        },
        Err(error) => {
            std::thread::sleep(std::time::Duration::from_secs(1));
            Err(error)
        }
    }
}
