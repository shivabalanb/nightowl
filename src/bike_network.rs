use std::{collections::HashMap, error::Error, fs::File, path::Path};

use serde::Deserialize;

use crate::{
    graph::Graph,
    ingestor::{Schedule, TransitStationDirectory},
    util::Coordinates,
};

// nearest dock, dock->destination
#[derive(Debug, Default)]
pub struct BikeNetwork {
    pub stations: HashMap<String, BikeStation>,
}

impl BikeNetwork {
    pub fn new() -> Self {
        Self::default()
    }
    /// Loads GBFS JSON format
    pub fn load_from_gbfs<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        agency_prefix: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let response: GbfsResponse = serde_json::from_reader(file)?;

        let prefix_id = |id: &str| -> String {
            match agency_prefix {
                Some(p) if !p.is_empty() => format("{}:{}", p, id),
                _ => id.to_string(),
            }
        };

        for raw in response.data.stations {
            let station_id = prefix_id(&raw.station_id);
            let station = BikeStation {
                id: station_id.clone(),
                name: raw.name,
                coordinates: Coordinates::new(raw.lat, raw.lon),
                capacity: raw.capacity,
            };
            self.stations.insert(station_id, station);
        }
        Ok(())
    }

    /// Convenience constructor
    pub fn from_gbfs<P: AsRef<Path>>(
        dir: P,
        agency_prefix: Option<&str>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut network = Self::new();
        network.load_from_gbfs(dir, agency_prefix)?;
        Ok(network)
    }
}

#[derive(Deserialize)]
struct GbfsResponse {
    data: GbfsData,
}

#[derive(Deserialize)]
struct GbfsData {
    stations: Vec<BikeStationRaw>,
}

#[derive(Deserialize, Clone)]
pub struct BikeStationRaw {
    pub station_id: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub capacity: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BikeStation {
    pub id: String,
    pub name: String,
    pub coordinates: Coordinates,
    pub capacity: Option<u32>,
}
