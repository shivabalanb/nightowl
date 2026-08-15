use std::collections::HashMap;

use crate::{
    ingestor::{Segment, SegmentDetail, StationDirectory},
    util::{Location, Time},
};

#[derive(Debug)]
pub struct Edge {
    pub from: Location,
    pub to: Location,
    pub departures: Vec<SegmentDetail>,
}

impl Edge {
    // returns earliest departure
    pub fn next_departure(&self, current_time: Time) -> Option<&SegmentDetail> {
        let idx = self
            .departures
            .partition_point(|d| d.departure_time < current_time);
        self.departures.get(idx)
    }
}

#[derive(Debug)]
pub struct Graph {
    pub adjacency_list: HashMap<Location, Vec<Edge>>,
}

impl Graph {
    pub fn from_segments(segments: Vec<Segment>, station_dir: &StationDirectory) -> Self {
        let mut adjacency_list: HashMap<Location, Vec<Edge>> = HashMap::new();

        for segment in segments {
            let from_loc = match station_dir.get_location(&segment.from_station_id) {
                Some(loc) => loc,
                None => continue,
            };
            let to_loc = match station_dir.get_location(&segment.to_station_id) {
                Some(loc) => loc,
                None => continue,
            };

            let edges = adjacency_list.entry(from_loc.clone()).or_default();

            if let Some(existing_edge) = edges.iter_mut().find(|e| e.to == to_loc) {
                existing_edge
                    .departures
                    .push(segment.transit_segment_detail);
            } else {
                edges.push(Edge {
                    from: from_loc,
                    to: to_loc,
                    departures: vec![segment.transit_segment_detail],
                });
            }
        }

        for edges in adjacency_list.values_mut() {
            for edge in edges {
                edge.departures.sort_by_key(|d| d.departure_time);
            }
        }

        Graph { adjacency_list }
    }
}
