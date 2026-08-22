use std::{
    collections::{HashMap, HashSet},
    error::Error,
    path::Path,
    str::FromStr,
};

use serde::Deserialize;

use crate::{
    ingestor::{Schedule, TransitStationDirectory},
    util::{Location, Time},
};

#[derive(Debug, Deserialize)]
struct StopTimeRow {
    pub trip_id: String,
    pub arrival_time: String,
    pub departure_time: String,
    pub stop_id: String,
    pub stop_sequence: u32,
}

#[derive(Debug, Clone)]
pub struct Departure {
    pub trip_id: String,
    pub service_id: String,
    pub departure_time: Time,
    pub travel_time: Time,
}

#[derive(Debug)]
pub struct Edge {
    pub to: Location,
    pub departures: Vec<Departure>,
}

impl Edge {
    pub fn next_departure(
        &self,
        current_time: Time,
        active_services: &HashSet<String>,
    ) -> Option<&Departure> {
        self.departures
            .iter()
            .filter(|d| d.departure_time >= current_time && active_services.contains(&d.service_id))
            .min_by_key(|d| d.departure_time)
    }
}

#[derive(Debug)]
pub struct Graph {
    pub adjacency_list: HashMap<Location, Vec<Edge>>,
}

impl Graph {
    pub fn from_gtfs_dir<P: AsRef<Path>>(
        dir: P,
        station_dir: &TransitStationDirectory,
        schedule: &Schedule,
    ) -> Result<Self, Box<dyn Error>> {
        Self::from_gtfs_file(dir.as_ref().join("stop_times.txt"), station_dir, schedule)
    }

    pub fn from_gtfs_file<P: AsRef<Path>>(
        stop_times_path: P,
        station_dir: &TransitStationDirectory,
        schedule: &Schedule,
    ) -> Result<Self, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(stop_times_path)?;
        let mut trips: HashMap<String, Vec<(u32, String, Time, Time)>> = HashMap::new();

        for result in rdr.deserialize() {
            let row: StopTimeRow = result?;
            if let (Ok(arr), Ok(dep)) = (
                Time::from_str(&row.arrival_time),
                Time::from_str(&row.departure_time),
            ) {
                let station_id = station_dir
                    .resolve_stop_to_station(&row.stop_id)
                    .unwrap_or(&row.stop_id);
                trips
                    .entry(row.trip_id)
                    .or_default()
                    .push((row.stop_sequence, station_id.to_string(), arr, dep));
            }
        }

        let mut adjacency_list: HashMap<Location, Vec<Edge>> = HashMap::new();

        for (trip_id, mut stops) in trips {
            stops.sort_by_key(|s| s.0);
            let service_id = schedule.get_service_id(&trip_id).unwrap_or_default().to_string();

            for window in stops.windows(2) {
                let (_, ref from_id, _, from_dep) = window[0];
                let (_, ref to_id, to_arr, _) = window[1];

                if from_id == to_id {
                    continue;
                }

                let from_loc = match station_dir.get_location(from_id) {
                    Some(loc) => loc,
                    None => continue,
                };
                let to_loc = match station_dir.get_location(to_id) {
                    Some(loc) => loc,
                    None => continue,
                };

                let travel_time = to_arr.saturating_sub(from_dep);
                let departure = Departure {
                    trip_id: trip_id.clone(),
                    service_id: service_id.clone(),
                    departure_time: from_dep,
                    travel_time,
                };

                let edges = adjacency_list.entry(from_loc).or_default();
                if let Some(edge) = edges.iter_mut().find(|e| e.to == to_loc) {
                    edge.departures.push(departure);
                } else {
                    edges.push(Edge {
                        to: to_loc,
                        departures: vec![departure],
                    });
                }
            }
        }

        for edges in adjacency_list.values_mut() {
            for edge in edges {
                edge.departures.sort_by_key(|d| d.departure_time);
            }
        }

        Ok(Graph { adjacency_list })
    }
}
