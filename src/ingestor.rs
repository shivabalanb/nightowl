use std::{collections::HashMap, error::Error, fmt::Display, ops::Deref, path::Path, str::FromStr};

use serde::Deserialize;

use crate::util::{Coordinates, Time};

#[derive(Debug, Deserialize)]
pub struct TransitStopRaw {
    pub stop_id: String,
    pub stop_name: String,
    pub stop_lat: f64,
    pub stop_lon: f64,
    #[serde(default)]
    pub parent_station: String,
}

#[derive(Debug)]
pub struct TransitStop {
    pub raw: TransitStopRaw,
    pub coordinates: Coordinates,
}

impl From<TransitStopRaw> for TransitStop {
    fn from(raw: TransitStopRaw) -> Self {
        let coordinates = Coordinates::new(raw.stop_lat, raw.stop_lon);
        Self { raw, coordinates }
    }
}

impl Deref for TransitStop {
    type Target = TransitStopRaw;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

#[derive(Debug)]
pub struct StopDirectory {
    directory: HashMap<String, TransitStop>,
    parent_map: HashMap<String, String>,
}

impl StopDirectory {
    pub fn new(stops: Vec<TransitStop>) -> Self {
        let mut directory = HashMap::new();
        let mut parent_map = HashMap::new();

        for stop in stops {
            if !stop.parent_station.is_empty() {
                parent_map.insert(stop.stop_id.clone(), stop.parent_station.clone());
            }
            directory.insert(stop.stop_id.clone(), stop);
        }
        StopDirectory {
            directory,
            parent_map,
        }
    }

    pub fn get_name(&self, stop_id: &str) -> &str {
        self.directory
            .get(stop_id)
            .map(|s| s.stop_name.as_str())
            .unwrap_or("Unknown Stop")
    }

    pub fn parent_map(&self) -> &HashMap<String, String> {
        &self.parent_map
    }
}

impl Display for StopDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n**LOAD TRANSIT STOPS**\n")?;
        let mut parent_stops: Vec<_> = self
            .directory
            .values()
            .filter(|s| s.parent_station.is_empty())
            .collect();
        parent_stops.sort_by_key(|s| &s.stop_id);

        for stop in parent_stops {
            writeln!(
                f,
                "stop: {} ({}) at ({:.4}, {:.4})",
                stop.stop_name, stop.stop_id, stop.stop_lat, stop.stop_lon
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
    pub from_stop_id: String,
    pub to_stop_id: String,
    pub transit_segment_detail: SegmentDetail,
}

pub fn load_transit_stops<P: AsRef<Path>>(
    file_path: P,
) -> Result<Vec<TransitStop>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut stops = Vec::with_capacity(100);

    for result in rdr.deserialize() {
        let stop_raw: TransitStopRaw = result?;
        stops.push(stop_raw.into());
    }
    Ok(stops)
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

pub fn group_stop_times_by_trip(
    stop_times: Vec<StopTime>,
) -> HashMap<String, Vec<StopTime>> {
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
                    from_stop_id: from_stop.stop_id.clone(),
                    to_stop_id: to_stop.stop_id.clone(),
                    transit_segment_detail,
                };
                transit_connections.push(transit_segment);
            };
        }
    }
    transit_connections
}
