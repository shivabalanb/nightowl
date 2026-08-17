use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use crate::{
    graph::Graph,
    ingestor::StationDirectory,
    util::{DateTime, Location, Time},
};

pub struct Query {
    pub origin: Location,
    pub destination: Location,
    pub departure_time: DateTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Leg {
    Transit {
        from: Location,
        to: Location,
        trip_id: String,
        departure_time: DateTime,
        arrival_time: DateTime,
        stops_count: usize,
    },
    Walk {
        from: Location,
        to: Location,
        distance_miles: f64,
        departure_time: DateTime,
        arrival_time: DateTime,
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

    pub fn departure_time(&self) -> DateTime {
        match self {
            Leg::Transit { departure_time, .. } => *departure_time,
            Leg::Walk { departure_time, .. } => *departure_time,
        }
    }

    pub fn arrival_time(&self) -> DateTime {
        match self {
            Leg::Transit { arrival_time, .. } => *arrival_time,
            Leg::Walk { arrival_time, .. } => *arrival_time,
        }
    }
}

pub fn merge_consecutive_transit_legs(legs: Vec<Leg>) -> Vec<Leg> {
    let mut merged: Vec<Leg> = Vec::new();

    for leg in legs {
        if let Some(Leg::Transit {
            to,
            arrival_time,
            trip_id,
            stops_count,
            ..
        }) = merged.last_mut()
        {
            if let Leg::Transit {
                to: next_to,
                arrival_time: next_arr,
                trip_id: next_trip_id,
                ..
            } = &leg
            {
                if trip_id == next_trip_id {
                    *to = next_to.clone();
                    *arrival_time = *next_arr;
                    *stops_count += 1;
                    continue;
                }
            }
        }
        merged.push(leg);
    }

    merged
}

#[derive(Debug)]
pub struct Plan {
    pub origin: Location,
    pub destination: Location,
    pub departure_time: DateTime,
    pub arrival_time: DateTime,
    pub legs: Vec<Leg>,
}

impl std::fmt::Display for Plan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total_mins = self
            .legs
            .last()
            .map(|last_leg| {
                let dep = self.departure_time.time.as_minutes();
                let arr = last_leg.arrival_time().time.as_minutes();
                arr.saturating_sub(dep)
            })
            .unwrap_or(0);

        writeln!(
            f,
            "===================================================================="
        )?;
        writeln!(
            f,
            "  ROUTE: {} ➔ {}",
            self.origin.name(),
            self.destination.name()
        )?;
        writeln!(
            f,
            "  Date:           {} ({})",
            self.departure_time.date,
            self.departure_time.date.day_of_week()
        )?;
        writeln!(f, "  Departure:      {}", self.departure_time.time)?;
        writeln!(f, "  Arrival:        {}", self.arrival_time.time)?;
        writeln!(f, "  Total Duration: {} mins", total_mins)?;
        writeln!(
            f,
            "===================================================================="
        )?;

        for (i, leg) in self.legs.iter().enumerate() {
            // Check for wait time before this leg
            if i > 0 {
                let prev_arr = self.legs[i - 1].arrival_time().time.as_minutes();
                let curr_dep = leg.departure_time().time.as_minutes();
                let wait_mins = curr_dep.saturating_sub(prev_arr);

                if wait_mins > 0 {
                    writeln!(f, "   ⏳ Wait {} mins at {}", wait_mins, leg.from().name())?;
                    writeln!(
                        f,
                        "--------------------------------------------------------------------"
                    )?;
                }
            }

            match leg {
                Leg::Transit {
                    from,
                    to,
                    trip_id,
                    departure_time,
                    arrival_time,
                    stops_count,
                } => {
                    let ride_mins = arrival_time
                        .time
                        .as_minutes()
                        .saturating_sub(departure_time.time.as_minutes());
                    let stop_label = if *stops_count == 1 {
                        "1 stop".to_string()
                    } else {
                        format!("{} stops", stops_count)
                    };
                    writeln!(
                        f,
                        "Leg {}: [🚆 Transit ({}) - {} mins, {}]",
                        i + 1,
                        trip_id,
                        ride_mins,
                        stop_label
                    )?;
                    writeln!(f, "   Board:   {:32} @ {}", from.name(), departure_time.time)?;
                    writeln!(f, "   Alight:  {:32} @ {}", to.name(), arrival_time.time)?;
                }
                Leg::Walk {
                    from,
                    to,
                    distance_miles,
                    departure_time,
                    arrival_time,
                } => {
                    let walk_mins = arrival_time
                        .time
                        .as_minutes()
                        .saturating_sub(departure_time.time.as_minutes());
                    writeln!(
                        f,
                        "Leg {}: [🚶 Walk ({:.2} mi, {} mins)]",
                        i + 1,
                        distance_miles,
                        walk_mins
                    )?;
                    writeln!(f, "   Start:   {:32} @ {}", from.name(), departure_time.time)?;
                    writeln!(f, "   End:     {:32} @ {}", to.name(), arrival_time.time)?;
                }
            }

            if i + 1 < self.legs.len() {
                writeln!(
                    f,
                    "--------------------------------------------------------------------"
                )?;
            }
        }
        writeln!(
            f,
            "===================================================================="
        )
    }
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub current_location: Location,
    pub current_time: DateTime,
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
    let mut best_times: HashMap<Location, DateTime> = HashMap::new();

    let init_state = SearchState {
        current_location: query.origin.clone(),
        current_time: query.departure_time,
        path: Vec::new(),
    };
    pq.push(init_state);

    // Initial best times
    best_times.insert(query.origin.clone(), query.departure_time);
    let direct_walk_arr = query.departure_time + query.origin.walk_duration(&query.destination);
    best_times.insert(query.destination.clone(), direct_walk_arr);

    while let Some(state) = pq.pop() {
        // 1) Destination reached
        if state.current_location == query.destination {
            return Some(Plan {
                origin: query.origin,
                destination: query.destination,
                departure_time: query.departure_time,
                arrival_time: state.current_time,
                legs: merge_consecutive_transit_legs(state.path),
            });
        }

        // 2) Pruning
        if let Some(&best) = best_times.get(&state.current_location)
            && state.current_time > best
        {
            continue;
        }

        // 3) Explore edges
        // I - Walk to nearby stations
        let start_stations =
            station_dir.find_nearby_stations(&state.current_location.get_coordinates(), 1.0);
        for (station, station_dist_miles) in start_stations {
            let mut new_path = state.path.clone();
            let arrival_time = state.current_time + state.current_location.walk_duration(&station);

            if arrival_time < best_times.get(&station).copied().unwrap_or(DateTime {
                date: query.departure_time.date.next_day(),
                time: Time::MAX,
            }) {
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

        // II - Transit: Station to station (day-aware + 2-min boarding buffer)
        let active_services = graph
            .calendar
            .active_services_for_date(&state.current_time.date);

        if let Some(edges) = graph.adjacency_list.get(&state.current_location) {
            for edge in edges {
                // If we are already on this train, no boarding buffer needed
                let is_staying_on_train = match state.path.last() {
                    Some(Leg::Transit { trip_id, .. }) => {
                        edge.departures.iter().any(|d| &d.trip_id == trip_id)
                    }
                    _ => false,
                };

                let buffer = match &state.current_location {
                    Location::Station { id, .. } => station_dir
                        .get_station(id)
                        .map_or(Time::from_minutes(2), |s| s.boarding_buffer()),
                    _ => Time::from_minutes(2),
                };

                let min_dep_time = if is_staying_on_train {
                    state.current_time.time
                } else {
                    state.current_time.time + buffer
                };

                if let Some(departure) = edge.next_departure(min_dep_time, &active_services) {
                    let dep_datetime =
                        DateTime::new(state.current_time.date, departure.departure_time);
                    let arrival_time = dep_datetime + departure.travel_time;

                    if arrival_time < best_times.get(&edge.to).copied().unwrap_or(DateTime {
                        date: query.departure_time.date.next_day(),
                        time: Time::MAX,
                    }) {
                        best_times.insert(edge.to.clone(), arrival_time);

                        let mut new_path = state.path.clone();
                        new_path.push(Leg::Transit {
                            from: state.current_location.clone(),
                            to: edge.to.clone(),
                            trip_id: departure.trip_id.clone(),
                            departure_time: dep_datetime,
                            arrival_time,
                            stops_count: 1,
                        });

                        pq.push(SearchState {
                            current_time: arrival_time,
                            current_location: edge.to.clone(),
                            path: new_path,
                        });
                    }
                }
            }
        }

        // III - Walk to final destination
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
