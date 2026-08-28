use std::{collections::HashMap, error::Error, time::Duration};
use serde::Deserialize;

pub const CITIBIKE_STATUS_URL: &str = "https://gbfs.citibikenyc.com/gbfs/en/station_status.json";

#[derive(Debug, Deserialize, Clone)]
pub struct GbfsStatusResponse {
    pub data: GbfsStatusData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GbfsStatusData {
    pub stations: Vec<GbfsStationStatusRaw>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GbfsStationStatusRaw {
    pub station_id: String,
    pub num_bikes_available: u32,
    pub num_ebikes_available: Option<u32>,
    pub num_docks_available: u32,
    pub is_renting: Option<u32>,
    pub is_returning: Option<u32>,
    pub is_installed: Option<u32>,
    pub last_reported: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct BikeStationStatus {
    pub num_bikes_available: u32,
    pub num_ebikes_available: u32,
    pub num_docks_available: u32,
    pub is_renting: bool,
    pub is_returning: bool,
}

impl BikeStationStatus {
    pub fn can_unlock(&self) -> bool {
        self.is_renting && self.num_bikes_available > 0
    }

    pub fn can_dock(&self) -> bool {
        self.is_returning && self.num_docks_available > 0
    }
}

/// Fetches live real-time Citi Bike dock availability
pub fn fetch_citibike_status() -> Result<HashMap<String, BikeStationStatus>, Box<dyn Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(4))
        .build()?;

    let resp: GbfsStatusResponse = client.get(CITIBIKE_STATUS_URL).send()?.json()?;
    let mut status_map = HashMap::with_capacity(resp.data.stations.len());

    for s in resp.data.stations {
        let is_renting = s.is_renting.unwrap_or(1) == 1 && s.is_installed.unwrap_or(1) == 1;
        let is_returning = s.is_returning.unwrap_or(1) == 1 && s.is_installed.unwrap_or(1) == 1;

        status_map.insert(
            s.station_id,
            BikeStationStatus {
                num_bikes_available: s.num_bikes_available,
                num_ebikes_available: s.num_ebikes_available.unwrap_or(0),
                num_docks_available: s.num_docks_available,
                is_renting,
                is_returning,
            },
        );
    }

    Ok(status_map)
}
