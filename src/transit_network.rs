use std::{error::Error, path::Path};

use serde::Deserialize;

use crate::{
    graph::Graph, ingestor::{Schedule, TransitStationDirectory}, util::Coordinates,
};

#[derive(Debug)]
pub struct TransitNetwork {
    pub stations: TransitStationDirectory,
    pub schedule: Schedule,
    pub graph: Graph,
}

impl Default for TransitNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl TransitNetwork {
    pub fn new() -> Self {
        Self {
            stations: TransitStationDirectory::new(),
            schedule: Schedule::new(),
            graph: Graph {
                adjacency_list: Default::default(),
            },
        }
    }

    /// Loads and merges GTFS data (stops, calendar, trips, stop_times)
    pub fn load_gtfs<P: AsRef<Path>>(
        &mut self,
        dir: P,
        agency_prefix: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let dir_path = dir.as_ref();
        let stops_path = dir_path.join("stops.txt");
        let calendar_path = dir_path.join("calendar.txt");
        let trips_path = dir_path.join("trips.txt");
        let stop_times_path = dir_path.join("stop_times.txt");

        self.stations.load_from_gtfs(stops_path, agency_prefix)?;
        self.schedule.load_from_gtfs(calendar_path, trips_path)?;

        let feed_graph = Graph::from_gtfs_file(stop_times_path, &self.stations, &self.schedule)?;
        for (location, edges) in feed_graph.adjacency_list {
            self.graph
                .adjacency_list
                .entry(location)
                .or_default()
                .extend(edges);
        }

        Ok(())
    }

    /// Convenience constructor
    pub fn from_gtfs_dir<P: AsRef<Path>>(
        dir: P,
        agency_prefix: Option<&str>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut network = Self::new();
        network.load_gtfs(dir, agency_prefix)?;
        Ok(network)
    }
}
#[derive(Debug, Deserialize)]
pub struct BikeStationRaw {
    pub stop_id: String,
    pub stop_name: String,
    pub stop_lat: f64,
    pub stop_lon: f64,
    #[serde(default)]
    pub parent_station: String,
}

#[derive(Debug, Clone)]
pub struct BikeStation {
    pub id: String,
    pub name: String,
    pub coordinates: Coordinates,
    pub capacity: Option<u32>,
}
