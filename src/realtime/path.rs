use std::{error::Error, time::Duration};
use serde::Deserialize;

pub const RIDE_PATH_URL: &str = "https://www.panynj.gov/bin/portauthority/ridepath.json";

#[derive(Debug, Deserialize, Clone)]
pub struct RidePathResponse {
    pub results: Vec<StationRealtime>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StationRealtime {
    #[serde(rename = "consideredStation")]
    pub considered_station: String,
    #[serde(default)]
    pub destinations: Vec<RealtimeDestination>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RealtimeDestination {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub messages: Vec<RealtimeMessage>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RealtimeMessage {
    #[serde(default)]
    pub target: String,
    #[serde(rename = "secondsToArrival")]
    pub seconds_to_arrival: String,
    #[serde(rename = "arrivalTimeMessage")]
    pub arrival_time_message: Option<String>,
    #[serde(rename = "headSign")]
    pub head_sign: Option<String>,
    #[serde(rename = "lineColor")]
    pub line_color: Option<String>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LiveTrainDeparture {
    pub station_code: String,
    pub station_name: String,
    pub destination_code: String,
    pub head_sign: String,
    pub seconds_to_arrival: u64,
    pub arrival_time_msg: String,
}

/// Maps PATH station abbreviations to human-readable names matching GTFS
pub fn station_code_to_name(code: &str) -> &'static str {
    match code.to_uppercase().as_str() {
        "NEW" => "Newport",
        "EXP" => "Exchange Place",
        "WTC" => "World Trade Center",
        "HOB" => "Hoboken",
        "JSQ" => "Journal Square",
        "GRV" => "Grove Street",
        "NWK" => "Newark",
        "HAR" => "Harrison",
        "CHR" => "Christopher Street",
        "9ST" => "9th Street",
        "14S" => "14th Street",
        "23S" => "23rd Street",
        "33S" => "33rd Street",
        _ => "Unknown",
    }
}

/// Fetches live real-time PATH departures
pub fn fetch_path_realtime() -> Result<Vec<LiveTrainDeparture>, Box<dyn Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?;

    let resp: RidePathResponse = client.get(RIDE_PATH_URL).send()?.json()?;
    let mut departures = Vec::new();

    for station in resp.results {
        let station_name = station_code_to_name(&station.considered_station).to_string();
        for dest in station.destinations {
            for msg in dest.messages {
                let seconds = msg.seconds_to_arrival.parse::<u64>().unwrap_or(0);
                departures.push(LiveTrainDeparture {
                    station_code: station.considered_station.clone(),
                    station_name: station_name.clone(),
                    destination_code: msg.target.clone(),
                    head_sign: msg.head_sign.unwrap_or_else(|| msg.target.clone()),
                    seconds_to_arrival: seconds,
                    arrival_time_msg: msg
                        .arrival_time_message
                        .unwrap_or_else(|| format!("{}s", seconds)),
                });
            }
        }
    }

    Ok(departures)
}
