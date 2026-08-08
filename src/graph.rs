use std::collections::HashMap;

use crate::{
    ingestor::{Segment, SegmentDetail},
    util::Time,
};

#[derive(Debug)]
pub struct Edge {
    pub from_stop_id: String,
    pub to_stop_id: String,
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
    pub adjacency_list: HashMap<String, Vec<Edge>>,
}

impl Graph {
    pub fn from_segments(segments: Vec<Segment>) -> Self {
        let mut adjacency_list: HashMap<String, Vec<Edge>> = HashMap::new();

        for segment in segments {
            let edges = adjacency_list.entry(segment.from_stop_id.clone()).or_default();
            if let Some(existing_edge) = edges
                .iter_mut()
                .find(|e| e.to_stop_id == segment.to_stop_id)
            {
                existing_edge
                    .departures
                    .push(segment.transit_segment_detail);
            } else {
                edges.push(Edge {
                    from_stop_id: segment.from_stop_id,
                    to_stop_id: segment.to_stop_id,
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
