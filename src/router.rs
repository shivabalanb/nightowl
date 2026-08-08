use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use crate::{
    graph::Graph,
    ingestor::StopDirectory,
    util::{Coordinates, Time},
};

pub struct Query {
    pub origin: Coordinates,
    pub destination: Coordinates,
    pub departure_time: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Leg {
    Transit {
        from_stop_id: String,
        to_stop_id: String,
        trip_id: String,
        departure_time: Time,
        arrival_time: Time,
    },
    Walk {
        departure_time: Time,
        arrival_time: Time,
    },
}

#[derive(Debug)]
pub struct Plan {
    pub travel_time: Time,
    pub origin_id: String,
    pub destination_id: String,
    pub legs: Vec<Leg>,
}

pub struct PlanDisplay<'a> {
    pub plan: &'a Plan,
    pub stops: &'a StopDirectory,
}

impl Plan {
    pub fn display<'a>(&'a self, stops: &'a StopDirectory) -> PlanDisplay<'a> {
        PlanDisplay { plan: self, stops }
    }
}

impl<'a> std::fmt::Display for PlanDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let origin_name = self.stops.get_name(&self.plan.origin_id);
        let dest_name = self.stops.get_name(&self.plan.destination_id);

        writeln!(
            f,
            "============================================================"
        )?;
        writeln!(
            f,
            "  ROUTE: {} ({}) ➔ {} ({})",
            origin_name, self.plan.origin_id, dest_name, self.plan.destination_id
        )?;
        writeln!(
            f,
            "  Total Duration: {} minutes",
            self.plan.travel_time.as_minutes()
        )?;
        writeln!(
            f,
            "============================================================"
        )?;

        for (i, leg) in self.plan.legs.iter().enumerate() {
            match leg {
                Leg::Transit {
                    from_stop_id,
                    to_stop_id,
                    trip_id,
                    departure_time,
                    arrival_time,
                } => {
                    let from_name = self.stops.get_name(from_stop_id);
                    let to_name = self.stops.get_name(to_stop_id);

                    writeln!(f, "Leg {}: [Trip ID: {}]", i + 1, trip_id)?;
                    writeln!(f, "   Get On:  {:30} @ {}", from_name, departure_time)?;
                    writeln!(f, "   Get Off: {:30} @ {}", to_name, arrival_time)?;
                }
                Leg::Walk {
                    departure_time,
                    arrival_time,
                } => {
                    writeln!(f, "Leg {}: [🚶 Walk]", i + 1)?;
                    writeln!(f, "   Get On:  {:30} @ {}", "Walk", departure_time)?;
                    writeln!(f, "   Get Off: {:30} @ {}", "Walk", arrival_time)?;
                }
            }
            if i + 1 < self.plan.legs.len() {
                writeln!(
                    f,
                    "------------------------------------------------------------"
                )?;
            }
        }
        writeln!(
            f,
            "============================================================"
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchState {
    pub current_stop_id: String,
    pub current_time: Time,
    pub path: Vec<Leg>,
}

impl Ord for SearchState {
    fn cmp(&self, other: &Self) -> Ordering {
        other.current_time.cmp(&self.current_time)
    }
}

impl PartialOrd for SearchState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn find_route(
    graph: &Graph,
    origin_id: &str,
    destination_id: &str,
    departure_time: Time,
) -> Option<Plan> {
    let mut pq = BinaryHeap::new();
    let mut best_times: HashMap<String, Time> = HashMap::new();

    let init_state = SearchState {
        current_stop_id: origin_id.to_string(),
        current_time: departure_time,
        path: Vec::new(),
    };
    pq.push(init_state);
    best_times.insert(origin_id.to_string(), departure_time);

    while let Some(state) = pq.pop() {
        // 1) check if at destination
        if state.current_stop_id == destination_id {
            return Some(Plan {
                origin_id: origin_id.to_string(),
                destination_id: destination_id.to_string(),
                travel_time: state.current_time - departure_time,
                legs: state.path,
            });
        }
        // 2) prune inefficient path
        if let Some(&best) = best_times.get(&state.current_stop_id)
            && state.current_time > best
        {
            continue;
        }
        // 3) explore outgoing edges
        if let Some(edges) = graph.adjacency_list.get(&state.current_stop_id) {
            for edge in edges {
                if let Some(departure) = edge.next_departure(state.current_time) {
                    let arrival_time = departure.departure_time + departure.travel_time;
                    if arrival_time
                        < best_times
                            .get(&edge.to_stop_id)
                            .copied()
                            .unwrap_or(Time::MAX)
                    {
                        best_times.insert(edge.to_stop_id.clone(), arrival_time);

                        let mut new_path = state.path.clone();
                        new_path.push(Leg::Transit {
                            from_stop_id: state.current_stop_id.clone(),
                            to_stop_id: edge.to_stop_id.clone(),
                            trip_id: departure.trip_id.clone(),
                            departure_time: departure.departure_time,
                            arrival_time,
                        });

                        pq.push(SearchState {
                            current_time: arrival_time,
                            current_stop_id: edge.to_stop_id.clone(),
                            path: new_path,
                        });
                    }
                };
            }
        }
    }

    None
}
