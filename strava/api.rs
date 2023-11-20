use std::collections::HashMap;

const BASE_URL: &str = "https://www.strava.com/api/v3";

type APIResponse = Result<String, reqwest::Error>;

pub fn get_response(path: &str, params: &str, token: &str) -> APIResponse {
    let response = reqwest::blocking::Client::new()
        .get(BASE_URL.to_owned() + path + params)
        .bearer_auth(&token)
        .send();

    match response {
        Ok(response) => Ok(String::from(&response.text().unwrap())),
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

/// First time user authentication
/// create the auth url
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

/// get the access and refresh tokens
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
        Ok(response) => Ok(String::from(&response.text().unwrap())),
        Err(err) => {
            println!("There was an error getting your activities, please try again.");
            Err(err)
        }
    }
}

/// Requires user refresh_token, client_id and client_secret
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
        Ok(response) => Ok(String::from(&response.text().unwrap())),
        Err(err) => {
            println!("There was an error getting your activities, please try again.");
            Err(err)
        }
    }
}
