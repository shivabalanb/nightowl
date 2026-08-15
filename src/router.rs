use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use crate::{
    graph::Graph,
    ingestor::StationDirectory,
    util::{Location, Time},
};

pub struct Query {
    pub origin: Location,
    pub destination: Location,
    pub departure_time: Time,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Leg {
    Transit {
        from: Location,
        to: Location,
        trip_id: String,
        departure_time: Time,
        arrival_time: Time,
    },
    Walk {
        from: Location,
        to: Location,
        distance_miles: f64,
        departure_time: Time,
        arrival_time: Time,
    },
}

impl Leg {
    pub fn from(&self) -> &Location {
        match self {
            Leg::Transit { from, .. } => from,
            Leg::Walk { from, .. } => from,
        }
    }

    pub fn to(&self) -> &Location {
        match self {
            Leg::Transit { to, .. } => to,
            Leg::Walk { to, .. } => to,
        }
    }
}

#[derive(Debug)]
pub struct Plan {
    pub travel_time: Time,
    pub origin: Location,
    pub destination: Location,
    pub legs: Vec<Leg>,
}

impl std::fmt::Display for Plan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dep_time = self.legs.first().map(|l| match l {
            Leg::Transit { departure_time, .. } => *departure_time,
            Leg::Walk { departure_time, .. } => *departure_time,
        });

        writeln!(
            f,
            "============================================================"
        )?;
        writeln!(
            f,
            "  ROUTE: {} ➔ {}",
            self.origin.name(),
            self.destination.name()
        )?;
        if let Some(dep) = dep_time {
            writeln!(f, "  Departure:      {}", dep)?;
        }
        writeln!(
            f,
            "  Total Duration: {} mins",
            self.travel_time.as_minutes()
        )?;
        writeln!(
            f,
            "============================================================"
        )?;

        for (i, leg) in self.legs.iter().enumerate() {
            match leg {
                Leg::Transit {
                    from,
                    to,
                    trip_id,
                    departure_time,
                    arrival_time,
                } => {
                    writeln!(f, "Leg {}: [🚆 Transit - Trip ID: {}]", i + 1, trip_id)?;
                    writeln!(f, "   Get On:  {:30} @ {}", from.name(), departure_time)?;
                    writeln!(f, "   Get Off: {:30} @ {}", to.name(), arrival_time)?;
                }
                Leg::Walk {
                    from,
                    to,
                    distance_miles,
                    departure_time,
                    arrival_time,
                } => {
                    let walk_mins = (*arrival_time - *departure_time).as_minutes();
                    writeln!(
                        f,
                        "Leg {}: [🚶 Walk ({:.2} mi, {} mins)]",
                        i + 1,
                        distance_miles,
                        walk_mins
                    )?;
                    writeln!(f, "   Start:   {:30} @ {}", from.name(), departure_time)?;
                    writeln!(f, "   End:     {:30} @ {}", to.name(), arrival_time)?;
                }
            }
            if i + 1 < self.legs.len() {
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

#[derive(Debug, Clone)]
pub struct SearchState {
    pub current_location: Location,
    pub current_time: Time,
    pub path: Vec<Leg>,
}

impl PartialEq for SearchState {
    fn eq(&self, other: &Self) -> bool {
        self.current_time == other.current_time
    }
}

impl Eq for SearchState {}

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

pub fn find_route(graph: &Graph, station_dir: &StationDirectory, query: Query) -> Option<Plan> {
    let mut pq = BinaryHeap::new();
    let mut best_times: HashMap<Location, Time> = HashMap::new();

    let init_state = SearchState {
        current_location: query.origin.clone(),
        current_time: query.departure_time,
        path: Vec::new(),
    };
    pq.push(init_state);
    // time to reach start
    best_times.insert(query.origin.clone(), query.departure_time);
    // time to beat - just walking
    best_times.insert(
        query.destination.clone(),
        query.departure_time + query.origin.walk_duration(&query.destination),
    );

    while let Some(state) = pq.pop() {
        // 1) check if at destination
        if state.current_location == query.destination {
            return Some(Plan {
                origin: query.origin,
                destination: query.destination,
                travel_time: state.current_time - query.departure_time,
                legs: state.path,
            });
        }
        // 2) prune inefficient path
        if let Some(&best) = best_times.get(&state.current_location)
            && state.current_time > best
        {
            continue;
        }
        // 3) explore outgoing edges
        // I - walk to nearby station
        let start_stations =
            station_dir.find_nearby_stations(&state.current_location.get_coordinates(), 1.0);
        for (station, station_dist_miles) in start_stations {
            let mut new_path = state.path.clone();
            let arrival_time = state.current_time + state.current_location.walk_duration(&station);
            if arrival_time < best_times.get(&station).copied().unwrap_or(Time::MAX) {
                best_times.insert(station.clone(), arrival_time);

                new_path.push(Leg::Walk {
                    from: state.current_location.clone(),
                    to: station.clone(),
                    distance_miles: station_dist_miles,
                    departure_time: state.current_time,
                    arrival_time,
                });

                pq.push(SearchState {
                    current_time: arrival_time,
                    current_location: station.clone(),
                    path: new_path,
                });
            }
        }

        // II - station to station
        if let Some(edges) = graph.adjacency_list.get(&state.current_location) {
            for edge in edges {
                if let Some(departure) = edge.next_departure(state.current_time) {
                    let arrival_time = departure.departure_time + departure.travel_time;
                    if arrival_time < best_times.get(&edge.to).copied().unwrap_or(Time::MAX) {
                        best_times.insert(edge.to.clone(), arrival_time);

                        let mut new_path = state.path.clone();
                        new_path.push(Leg::Transit {
                            from: state.current_location.clone(),
                            to: edge.to.clone(),
                            trip_id: departure.trip_id.clone(),
                            departure_time: departure.departure_time,
                            arrival_time,
                        });

                        pq.push(SearchState {
                            current_time: arrival_time,
                            current_location: edge.to.clone(),
                            path: new_path,
                        });
                    }
                };
            }
        }

        // III- walk to dest
        let mut new_path = state.path.clone();
        let arrival_time =
            state.current_time + state.current_location.walk_duration(&query.destination);

        if arrival_time < best_times.get(&query.destination).copied().unwrap() {
            best_times.insert(query.destination.clone(), arrival_time);
            new_path.push(Leg::Walk {
                from: state.current_location.clone(),
                to: query.destination.clone(),
                distance_miles: state.current_location.walk_miles(&query.destination),
                departure_time: state.current_time,
                arrival_time,
            });

            pq.push(SearchState {
                current_time: arrival_time,
                current_location: query.destination.clone(),
                path: new_path,
            });
        }
    }

    None
}
