use std::{error::Error, time::{Duration, Instant}};

pub const NJT_TOKEN_URL: &str = "https://raildata.njtransit.com/api/GTFSRT/getToken";
pub const NJT_GTFS_URL: &str = "https://raildata.njtransit.com/api/GTFSRT/getGTFS";

pub struct NjtClient {
    pub username: String,
    pub password: String,
    token: Option<String>,
    token_acquired_at: Option<Instant>,
}

impl NjtClient {
    pub fn new(username: String, password: String) -> Self {
        Self {
            username,
            password,
            token: None,
            token_acquired_at: None,
        }
    }

    /// Creates an NJ Transit client from environment variables `NJT_USERNAME` and `NJT_PASSWORD`
    pub fn from_env() -> Option<Self> {
        let username = std::env::var("NJT_USERNAME").ok()?;
        let password = std::env::var("NJT_PASSWORD").ok()?;
        if username.trim().is_empty() || password.trim().is_empty() {
            return None;
        }
        Some(Self::new(username, password))
    }

    /// Obtains a cached token or requests a new one if expired (valid for hours)
    pub fn get_token(&mut self) -> Result<String, Box<dyn Error>> {
        // Reuse cached token if less than 2 hours old
        if let (Some(token), Some(acquired)) = (&self.token, self.token_acquired_at) {
            if acquired.elapsed() < Duration::from_secs(2 * 3600) {
                return Ok(token.clone());
            }
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        let form = reqwest::blocking::multipart::Form::new()
            .text("username", self.username.clone())
            .text("password", self.password.clone());

        let res = client
            .post(NJT_TOKEN_URL)
            .multipart(form)
            .send()?
            .text()?;

        let clean_token = if let Ok(val) = serde_json::from_str::<serde_json::Value>(&res) {
            val.get("UserToken")
                .and_then(|t| t.as_str())
                .unwrap_or(&res)
                .to_string()
        } else {
            res.trim().trim_matches('"').to_string()
        };
        self.token = Some(clean_token.clone());
        self.token_acquired_at = Some(Instant::now());

        Ok(clean_token)
    }

    /// Fetches live GTFS data using the token
    pub fn fetch_gtfs_realtime(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let token = self.get_token()?;
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        let form = reqwest::blocking::multipart::Form::new().text("token", token);

        let res = client
            .post(NJT_GTFS_URL)
            .multipart(form)
            .send()?
            .bytes()?;

        Ok(res.to_vec())
    }
}
