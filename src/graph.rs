use std::collections::HashMap;

use crate::{
    ingestor::{TransitSegment, TransitSegmentDetail},
    util::Time,
};

#[derive(Debug)]
pub struct TransitEdgepoint {
    pub to_stop_id: String,
    pub departures: Vec<TransitSegmentDetail>,
}

impl TransitEdgepoint {
    // returns earliest departure
    pub fn next_departure(&self, current_time: Time) -> Option<&TransitSegmentDetail> {
        let idx = self
            .departures
            .partition_point(|d| d.departure_time < current_time);
        self.departures.get(idx)
    }
}

#[derive(Debug)]
pub struct TransitGraph {
    pub adjacency_list: HashMap<String, Vec<TransitEdgepoint>>,
}

impl TransitGraph {
    pub fn from_segments(segments: Vec<TransitSegment>) -> Self {
        let mut adjacency_list: HashMap<String, Vec<TransitEdgepoint>> = HashMap::new();

        for segment in segments {
            let edges = adjacency_list.entry(segment.from_stop_id).or_default();
            if let Some(existing_edge) = edges
                .iter_mut()
                .find(|e| e.to_stop_id == segment.to_stop_id)
            {
                existing_edge
                    .departures
                    .push(segment.transit_segment_detail);
            } else {
                edges.push(TransitEdgepoint {
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

        TransitGraph { adjacency_list }
    }
}
