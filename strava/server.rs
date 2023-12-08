use rocket::config::{Config, Environment, LoggingLevel};
use rocket::http::RawStr;
use rocket::{get, routes, State};
use std::sync::{mpsc, Mutex};

/// Represents the authentication information.
#[derive(Debug)]
pub struct AuthInfo {
    pub code: String,
    pub scopes: Vec<String>,
}

impl AuthInfo {
    /// Creates a new `AuthInfo` instance.
    ///
    /// # Arguments
    ///
    /// * `code` - The authentication code.
    /// * `scopes` - The list of scopes.
    ///
    /// # Example
    ///
    /// ```
    /// use strava::server::AuthInfo;
    /// use rocket::http::RawStr;
    ///
    /// let code = RawStr::from_str("12345");
    /// let scopes = RawStr::from_str("scope1,scope2");
    /// let auth_info = AuthInfo::new(&code, &scopes);
    /// ```
    pub fn new(code: &RawStr, scopes: &RawStr) -> Self {
        Self {
            code: String::from(code.as_str()),
            scopes: scopes.as_str().split(',').map(String::from).collect(),
        }
    }
}

/// Represents the result of an authentication operation.
pub type AuthResult = Result<AuthInfo, String>;

/// Represents the transmitter for sending authentication results.
pub type Transmitter = mpsc::Sender<AuthResult>;

/// Represents the mutex for the transmitter.
pub type TxMutex<'req> = State<'req, Mutex<Transmitter>>;

// --

/// Handles the successful authentication request.
///
/// # Arguments
///
/// * `code` - The authentication code.
/// * `scope` - The list of scopes.
/// * `tx_mutex` - The mutex for the transmitter.
///
/// # Returns
///
/// A string indicating the success message.
#[get("/?<code>&<scope>")]
fn success(code: &RawStr, scope: &RawStr, tx_mutex: TxMutex) -> &'static str {
    let tx = tx_mutex.lock().unwrap();
    tx.send(Ok(AuthInfo::new(code, scope))).unwrap();
    "✅ You may close this browser tab and return to the terminal."
}

/// Handles the error in the authentication request.
///
/// # Arguments
///
/// * `error` - The error message.
/// * `tx_mutex` - The mutex for the transmitter.
///
/// # Returns
///
/// A string indicating the error message.
#[get("/?<error>", rank = 2)]
fn error(error: &RawStr, tx_mutex: TxMutex) -> String {
    let tx = tx_mutex.lock().unwrap();
    tx.send(Err(String::from(error.as_str()))).unwrap();
    format!("Error: {}, please return to the terminal.", error)
}

// --

/// Starts the Rocket server.
///
/// # Arguments
///
/// * `tx` - The transmitter for sending authentication results.
pub fn start(tx: Transmitter) {
    let config = Config::build(Environment::Development)
        .log_level(LoggingLevel::Off)
        .workers(1)
        .finalize()
        .unwrap();
    rocket::custom(config)
        .mount("/", routes![success, error])
        .manage(Mutex::new(tx))
        .launch();
}
