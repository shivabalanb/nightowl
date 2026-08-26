use std::{collections::HashMap, error::Error, fs::File, path::Path};

use serde::Deserialize;

use crate::util::{Coordinates, Location};

#[derive(Debug, Default)]
pub struct BikeNetwork {
    pub stations: HashMap<String, BikeStation>,
}

impl BikeNetwork {
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads GBFS JSON format (accepts directory containing station_information.json or file path)
    pub fn load_from_gbfs<P: AsRef<Path>>(
        &mut self,
        path: P,
        agency_prefix: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let p = path.as_ref();
        let file_path = if p.is_dir() {
            p.join("station_information.json")
        } else {
            p.to_path_buf()
        };
        let file = File::open(file_path)?;
        let response: GbfsResponse = serde_json::from_reader(file)?;

        let prefix_id = |id: &str| -> String {
            match agency_prefix {
                Some(p) if !p.is_empty() => format!("{}:{}", p, id),
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
                region_id: raw.region_id,
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

    /// Find nearby bike docks within `max_distance_miles` of a coordinate
    pub fn find_nearby_stations(
        &self,
        coords: &Coordinates,
        max_distance_miles: f64,
    ) -> Vec<(Location, f64)> {
        self.stations
            .values()
            .map(|s| {
                let location = s.to_location();
                let dist = coords.distance_to(&s.coordinates);
                (location, dist)
            })
            .filter(|(_, dist)| *dist <= max_distance_miles)
            .collect()
    }

    pub fn get_station(&self, station_id: &str) -> Option<&BikeStation> {
        self.stations.get(station_id)
    }

    pub fn get_location(&self, station_id: &str) -> Option<Location> {
        self.stations.get(station_id).map(|s| s.to_location())
    }

    /// Checks if biking between two docks is physically connected (e.g. not separated by the Hudson River)
    pub fn can_bike_between(&self, from: &Location, to: &Location) -> bool {
        let get_is_nj = |loc: &Location| -> bool {
            match loc {
                Location::Station { id, .. } => {
                    self.stations.get(id).map_or_else(
                        || loc.get_coordinates().lon < -74.025,
                        |s| s.is_nj(),
                    )
                }
                Location::Point(c) => c.lon < -74.025,
            }
        };

        get_is_nj(from) == get_is_nj(to)
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
    pub region_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BikeStation {
    pub id: String,
    pub name: String,
    pub coordinates: Coordinates,
    pub capacity: Option<u32>,
    pub region_id: Option<String>,
}

impl BikeStation {
    pub fn to_location(&self) -> Location {
        Location::Station {
            id: self.id.clone(),
            name: self.name.clone(),
            coords: self.coordinates,
        }
    }

    pub fn is_nj(&self) -> bool {
        match self.region_id.as_deref() {
            Some("70") | Some("311") => true, // 70 = Jersey City, 311 = Hoboken
            Some("71") => false,              // 71 = NYC (Manhattan, Brooklyn, Queens, Bronx)
            _ => self.coordinates.lon < -74.025,
        }
    }
}
