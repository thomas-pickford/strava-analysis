use reqwest::StatusCode;
use std::collections::HashMap;

const BASE_URL: &str = "https://www.strava.com/api/v3";

type APIResponse = Result<Response, reqwest::Error>;

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub body: String,
}

/// Sends a GET request to the specified API endpoint with the given parameters and authentication token.
/// 
/// # Arguments
/// 
/// * `path` - The path of the API endpoint.
/// * `params` - The parameters to be included in the request.
/// * `token` - The authentication token for the request.
/// 
/// # Returns
/// 
/// Returns an `APIResponse` containing the status code and body of the response if the request is successful,
/// otherwise returns an `Err` with the corresponding error.
pub fn get(path: &str, params: &str, token: &str) -> APIResponse {
    let response = reqwest::blocking::Client::new()
        .get(BASE_URL.to_owned() + path + params)
        .bearer_auth(&token)
        .send();

    match response {
        Ok(response) => Ok(Response {
            status: response.status(),
            body: String::from(&response.text().unwrap()),
        }),
        Err(err) => {
            println!("There was an error getting your activities, please try again.");
            Err(err)
        }
    }
}

// post
// fn post(url: &str, token: &str) -> APIResponse {
//     todo!();
// }

/// Generates the authorization URL for Strava API authentication.
///
/// # Arguments
///
/// * `client_id` - The client ID provided by Strava.
/// * `scopes` - The list of scopes required for the authentication.
///
/// # Returns
///
/// The authorization URL as a string.
pub fn auth_url(client_id: u32, scopes: &[&str]) -> String {
    let params = [
        format!("client_id={}", client_id),
        String::from("redirect_uri=http://localhost:8000"),
        String::from("response_type=code"),
        String::from("approval_prompt=auto"),
        format!("scope={}", scopes.join(",")),
    ]
    .join("&");

    format!("https://www.strava.com/oauth/authorize?{}", params)
}

/// Exchanges an authorization code for an access token using the Strava API.
///
/// # Arguments
///
/// * `code` - The authorization code obtained from the user.
/// * `id` - The client ID provided by Strava.
/// * `secret` - The client secret provided by Strava.
///
/// # Returns
///
/// Returns an `APIResponse` containing the status code and response body.
pub fn exchange_token(code: &str, id: u32, secret: &str) -> APIResponse {
    let mut body = HashMap::new();
    body.insert("client_id", format!("{}", id));
    body.insert("client_secret", String::from(secret));
    body.insert("code", String::from(code));
    body.insert("grant_type", String::from("authorization_code"));
    let response = reqwest::blocking::Client::new()
        .post("https://www.strava.com/oauth/token")
        .json(&body)
        .send();

    match response {
        Ok(response) => Ok(Response {
            status: response.status(),
            body: String::from(&response.text().unwrap()),
        }),
        Err(err) => {
            println!("There was an error getting your activities, please try again.");
            Err(err)
        }
    }
}

/// Refreshes the access token using the provided refresh token, client ID, and client secret.
/// 
/// # Arguments
/// 
/// * `refresh_token` - The refresh token used to obtain a new access token.
/// * `client_id` - The client ID associated with the application.
/// * `client_secret` - The client secret associated with the application.
/// 
/// # Returns
/// 
/// Returns an `APIResponse` containing the status code and response body.
pub fn refresh_token(refresh_token: &str, client_id: u32, client_secret: String) -> APIResponse {
    let mut body = HashMap::new();
    body.insert("client_id", format!("{}", client_id));
    body.insert("client_secret", String::from(client_secret));
    body.insert("grant_type", String::from("refresh_token"));
    body.insert("refresh_token", String::from(refresh_token));
    let response = reqwest::blocking::Client::new()
        .post("https://www.strava.com/oauth/token")
        .json(&body)
        .send();

    match response {
        Ok(response) => Ok(Response {
            status: response.status(),
            body: String::from(&response.text().unwrap()),
        }),
        Err(err) => {
            println!("There was an error getting your activities, please try again.");
            Err(err)
        }
    }
}
