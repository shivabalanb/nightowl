use std::{collections::HashMap, error::Error, fmt::Display, ops::Deref, path::Path, str::FromStr};

use serde::Deserialize;

use crate::util::{Coordinates, Location, Time};

#[derive(Debug, Deserialize)]
pub struct TransitStationRaw {
    #[serde(rename = "stop_id")]
    pub station_id: String,
    #[serde(rename = "stop_name")]
    pub station_name: String,
    #[serde(rename = "stop_lat")]
    pub station_lat: f64,
    #[serde(rename = "stop_lon")]
    pub station_lon: f64,
    #[serde(default)]
    pub parent_station: String,
}

#[derive(Debug)]
pub struct TransitStation {
    pub raw: TransitStationRaw,
    pub coordinates: Coordinates,
}

impl From<TransitStationRaw> for TransitStation {
    fn from(raw: TransitStationRaw) -> Self {
        let coordinates = Coordinates::new(raw.station_lat, raw.station_lon);
        Self { raw, coordinates }
    }
}

impl Deref for TransitStation {
    type Target = TransitStationRaw;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

#[derive(Debug)]
pub struct StationDirectory {
    directory: HashMap<String, TransitStation>,
    parent_map: HashMap<String, String>,
}

impl StationDirectory {
    pub fn new(stations: Vec<TransitStation>) -> Self {
        let mut directory = HashMap::new();
        let mut parent_map = HashMap::new();

        for station in stations {
            if !station.parent_station.is_empty() {
                parent_map.insert(station.station_id.clone(), station.parent_station.clone());
            }
            directory.insert(station.station_id.clone(), station);
        }
        StationDirectory {
            directory,
            parent_map,
        }
    }

    pub fn get_name(&self, station_id: &str) -> &str {
        self.directory
            .get(station_id)
            .map(|s| s.station_name.as_str())
            .unwrap_or("Unknown Station")
    }

    pub fn get_station(&self, station_id: &str) -> Option<&TransitStation> {
        self.directory.get(station_id)
    }

    pub fn get_location(&self, station_id: &str) -> Option<Location> {
        self.directory
            .get(station_id)
            .map(|station| Location::Station {
                id: station.station_id.clone(),
                name: station.station_name.clone(),
                coords: station.coordinates,
            })
    }

    pub fn find_nearby_stations(
        &self,
        coords: &Coordinates,
        max_distance_miles: f64,
    ) -> Vec<(Location, f64)> {
        self.directory
            .values()
            .filter(|s| s.parent_station.is_empty())
            .map(|s| {
                let location = self.get_location(&s.station_id).unwrap();
                let dist = coords.distance_to(&s.coordinates);
                (location, dist)
            })
            .filter(|(_, dist)| *dist <= max_distance_miles)
            .collect()
    }

    pub fn parent_map(&self) -> &HashMap<String, String> {
        &self.parent_map
    }
}

impl Display for StationDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n**LOAD TRANSIT STATIONS**\n")?;
        let mut parent_stations: Vec<_> = self
            .directory
            .values()
            .filter(|s| s.parent_station.is_empty())
            .collect();
        parent_stations.sort_by_key(|s| &s.station_id);

        for station in parent_stations {
            writeln!(
                f,
                "station: {} ({}) at ({:.4}, {:.4})",
                station.station_name, station.station_id, station.station_lat, station.station_lon
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct StopTime {
    pub trip_id: String,
    pub arrival_time: String,
    pub departure_time: String,
    pub stop_id: String,
    pub stop_sequence: u32,
}

#[derive(Debug, Clone)]
pub struct SegmentDetail {
    pub trip_id: String,
    pub departure_time: Time,
    pub travel_time: Time,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub from_station_id: String,
    pub to_station_id: String,
    pub transit_segment_detail: SegmentDetail,
}

pub fn load_transit_stations<P: AsRef<Path>>(
    file_path: P,
) -> Result<Vec<TransitStation>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut stations = Vec::with_capacity(100);

    for result in rdr.deserialize() {
        let station_raw: TransitStationRaw = result?;
        stations.push(station_raw.into());
    }
    Ok(stations)
}

pub fn load_transit_stop_times<P: AsRef<Path>>(
    file_path: P,
    parent_map: &HashMap<String, String>,
) -> Result<Vec<StopTime>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut stop_times = Vec::with_capacity(160_000);

    for result in rdr.deserialize() {
        let mut stop_time: StopTime = result?;
        if let Some(parent_id) = parent_map.get(&stop_time.stop_id) {
            stop_time.stop_id = parent_id.clone();
        }
        stop_times.push(stop_time);
    }
    Ok(stop_times)
}

pub fn group_stop_times_by_trip(stop_times: Vec<StopTime>) -> HashMap<String, Vec<StopTime>> {
    let mut trips: HashMap<String, Vec<StopTime>> = HashMap::new();

    for stop_time in stop_times {
        trips
            .entry(stop_time.trip_id.clone())
            .or_default()
            .push(stop_time);
    }

    for stops in trips.values_mut() {
        stops.sort_by_key(|s| s.stop_sequence);
    }

    trips
}

pub fn extract_transit_segments(
    grouped_stop_times: &HashMap<String, Vec<StopTime>>,
) -> Vec<Segment> {
    let mut transit_connections: Vec<Segment> = Vec::new();
    for (trip_id, stop_times) in grouped_stop_times {
        for pair in stop_times.windows(2) {
            let from_stop = &pair[0];
            let to_stop = &pair[1];

            if let (Ok(b_arr_time), Ok(a_dep_time)) = (
                Time::from_str(&to_stop.arrival_time),
                Time::from_str(&from_stop.departure_time),
            ) {
                let travel_time = b_arr_time.saturating_sub(a_dep_time);
                let transit_segment_detail = SegmentDetail {
                    trip_id: trip_id.clone(),
                    departure_time: a_dep_time,
                    travel_time,
                };
                let transit_segment = Segment {
                    from_station_id: from_stop.stop_id.clone(),
                    to_station_id: to_stop.stop_id.clone(),
                    transit_segment_detail,
                };
                transit_connections.push(transit_segment);
            };
        }
    }
    transit_connections
}
