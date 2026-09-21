use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use crate::{
    bike_network::BikeNetwork,
    transit_network::TransitNetwork,
    util::{DateTime, Location, ModeSet, Time, TransitMode},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Query {
    pub origin: Location,
    pub destination: Location,
    pub departure_time: Option<DateTime>,
    pub modes: ModeSet,
}

impl Query {
    pub fn new(origin: Location, destination: Location) -> Self {
        Self {
            origin,
            destination,
            departure_time: None,
            modes: ModeSet::all(),
        }
    }

    pub fn with_departure_time(mut self, departure_time: DateTime) -> Self {
        self.departure_time = Some(departure_time);
        self
    }

    pub fn with_modes(mut self, modes: ModeSet) -> Self {
        self.modes = modes;
        self
    }

    pub fn get_departure_time(&self) -> DateTime {
        self.departure_time.unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
    Bike {
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
            Leg::Bike { from, .. } => from,
        }
    }

    pub fn to(&self) -> &Location {
        match self {
            Leg::Transit { to, .. } => to,
            Leg::Walk { to, .. } => to,
            Leg::Bike { to, .. } => to,
        }
    }

    pub fn departure_time(&self) -> DateTime {
        match self {
            Leg::Transit { departure_time, .. } => *departure_time,
            Leg::Walk { departure_time, .. } => *departure_time,
            Leg::Bike { departure_time, .. } => *departure_time,
        }
    }

    pub fn arrival_time(&self) -> DateTime {
        match self {
            Leg::Transit { arrival_time, .. } => *arrival_time,
            Leg::Walk { arrival_time, .. } => *arrival_time,
            Leg::Bike { arrival_time, .. } => *arrival_time,
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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RouteStats {
    pub total_duration_mins: u32,
    pub walk_mins: u32,
    pub bike_mins: u32,
    pub transit_mins: u32,
    pub wait_mins: u32,
    pub walk_miles: f64,
    pub bike_miles: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Plan {
    pub origin: Location,
    pub destination: Location,
    pub departure_time: DateTime,
    pub arrival_time: DateTime,
    pub legs: Vec<Leg>,
}

impl Plan {
    pub fn stats(&self) -> RouteStats {
        let mut walk_mins = 0;
        let mut bike_mins = 0;
        let mut transit_mins = 0;
        let mut wait_mins = 0;
        let mut walk_dist = 0.0;
        let mut bike_dist = 0.0;

        for (i, leg) in self.legs.iter().enumerate() {
            if i > 0 {
                let prev_arr = self.legs[i - 1].arrival_time().time.as_minutes();
                let curr_dep = leg.departure_time().time.as_minutes();
                wait_mins += curr_dep.saturating_sub(prev_arr);
            }
            match leg {
                Leg::Walk {
                    distance_miles,
                    departure_time,
                    arrival_time,
                    ..
                } => {
                    walk_mins += arrival_time
                        .time
                        .as_minutes()
                        .saturating_sub(departure_time.time.as_minutes());
                    walk_dist += distance_miles;
                }
                Leg::Bike {
                    distance_miles,
                    departure_time,
                    arrival_time,
                    ..
                } => {
                    bike_mins += arrival_time
                        .time
                        .as_minutes()
                        .saturating_sub(departure_time.time.as_minutes());
                    bike_dist += distance_miles;
                }
                Leg::Transit {
                    departure_time,
                    arrival_time,
                    ..
                } => {
                    transit_mins += arrival_time
                        .time
                        .as_minutes()
                        .saturating_sub(departure_time.time.as_minutes());
                }
            }
        }

        let total_duration_mins = self
            .legs
            .last()
            .map(|l| {
                l.arrival_time()
                    .time
                    .as_minutes()
                    .saturating_sub(self.departure_time.time.as_minutes())
            })
            .unwrap_or(0);

        RouteStats {
            total_duration_mins,
            walk_mins,
            bike_mins,
            transit_mins,
            wait_mins,
            walk_miles: (walk_dist * 100.0).round() / 100.0,
            bike_miles: (bike_dist * 100.0).round() / 100.0,
        }
    }
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
            "ROUTE: {} -> {}",
            self.origin.name(),
            self.destination.name()
        )?;
        writeln!(
            f,
            "Date:  {} ({}) | Dep: {} | Arr: {} | Duration: {} mins",
            self.departure_time.date,
            self.departure_time.date.day_of_week(),
            self.departure_time.time,
            self.arrival_time.time,
            total_mins
        )?;
        writeln!(
            f,
            "===================================================================="
        )?;

        for (i, leg) in self.legs.iter().enumerate() {
            if i > 0 {
                let prev_arr = self.legs[i - 1].arrival_time().time.as_minutes();
                let curr_dep = leg.departure_time().time.as_minutes();
                let wait_mins = curr_dep.saturating_sub(prev_arr);

                if wait_mins > 0 {
                    writeln!(f, "  Wait {} mins at {}", wait_mins, leg.from().name())?;
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
                        "Leg {}: [Transit ({}) - {} mins, {}]",
                        i + 1,
                        trip_id,
                        ride_mins,
                        stop_label
                    )?;
                    writeln!(
                        f,
                        "  Board:   {:40} @ {}",
                        from.name(),
                        departure_time.time
                    )?;
                    writeln!(f, "  Alight:  {:40} @ {}", to.name(), arrival_time.time)?;
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

                    let walk_label = match (from.is_bike_dock(), to.is_transit_station(), to.is_bike_dock()) {
                        (true, true, _) => "Walk to Station",
                        (_, _, true) => "Walk to Bike Dock",
                        _ if i + 1 == self.legs.len() => "Walk to Destination",
                        _ => "Walk",
                    };

                    writeln!(
                        f,
                        "Leg {}: [{} ({:.2} mi, {} mins)]",
                        i + 1,
                        walk_label,
                        distance_miles,
                        walk_mins
                    )?;
                    writeln!(
                        f,
                        "  Start:   {:40} @ {}",
                        from.name(),
                        departure_time.time
                    )?;
                    writeln!(f, "  End:     {:40} @ {}", to.name(), arrival_time.time)?;
                }
                Leg::Bike {
                    from,
                    to,
                    distance_miles,
                    departure_time,
                    arrival_time,
                } => {
                    let bike_mins = arrival_time
                        .time
                        .as_minutes()
                        .saturating_sub(departure_time.time.as_minutes());
                    writeln!(
                        f,
                        "Leg {}: [Bike ({:.2} mi, {} mins)]",
                        i + 1,
                        distance_miles,
                        bike_mins
                    )?;
                    writeln!(
                        f,
                        "  Unlock:  {:40} @ {}",
                        from.name(),
                        departure_time.time
                    )?;
                    writeln!(f, "  Dock:    {:40} @ {}", to.name(), arrival_time.time)?;
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

impl SearchState {
    pub fn last_leg(&self) -> Option<&Leg> {
        self.path.last()
    }
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

struct SearchContext<'a> {
    transit: &'a TransitNetwork,
    bike: &'a BikeNetwork,
    query: &'a Query,
    departure_time: DateTime,
    pq: BinaryHeap<SearchState>,
    best_times: HashMap<Location, DateTime>,
}

impl<'a> SearchContext<'a> {
    fn new(transit: &'a TransitNetwork, bike: &'a BikeNetwork, query: &'a Query) -> Self {
        let departure_time = query.get_departure_time();
        let mut pq = BinaryHeap::new();
        let mut best_times = HashMap::new();

        best_times.insert(query.origin.clone(), departure_time);
        if bike.can_bike_between(&query.origin, &query.destination) {
            let direct_walk = departure_time + query.origin.walk_duration(&query.destination);
            best_times.insert(query.destination.clone(), direct_walk);
        }

        pq.push(SearchState {
            current_location: query.origin.clone(),
            current_time: departure_time,
            path: Vec::new(),
        });

        Self {
            transit,
            bike,
            query,
            departure_time,
            pq,
            best_times,
        }
    }

    fn try_step(
        &mut self,
        state: &SearchState,
        next_location: Location,
        arrival_time: DateTime,
        leg: Leg,
    ) {
        let best = self
            .best_times
            .get(&next_location)
            .copied()
            .unwrap_or(DateTime {
                date: self.departure_time.date.next_day(),
                time: Time::MAX,
            });

        if arrival_time < best {
            self.best_times.insert(next_location.clone(), arrival_time);
            let mut new_path = state.path.clone();
            new_path.push(leg);
            self.pq.push(SearchState {
                current_time: arrival_time,
                current_location: next_location,
                path: new_path,
            });
        }
    }

    fn is_bike_dock(&self, location: &Location) -> bool {
        match location {
            Location::Station { id, .. } => self.bike.get_station(id).is_some(),
            _ => false,
        }
    }

    // -------------------------------------------------------------------------
    // Edge Exploration Helpers
    // -------------------------------------------------------------------------

    fn explore_walk_to_transit(&mut self, state: &SearchState) {
        if matches!(state.last_leg(), Some(Leg::Walk { .. })) {
            return;
        }
        let stations = self
            .transit
            .stations
            .find_nearby_stations(&state.current_location.get_coordinates(), 1.0);

        for (station, dist_miles) in stations {
            if station == state.current_location {
                continue;
            }
            if !self.bike.can_bike_between(&state.current_location, &station) {
                continue;
            }
            let arrival_time = state.current_time + state.current_location.walk_duration(&station);
            self.try_step(
                state,
                station.clone(),
                arrival_time,
                Leg::Walk {
                    from: state.current_location.clone(),
                    to: station,
                    distance_miles: dist_miles,
                    departure_time: state.current_time,
                    arrival_time,
                },
            );
        }
    }

    fn explore_walk_to_bike_docks(&mut self, state: &SearchState) {
        if !self.query.modes.allows(TransitMode::Bike) {
            return;
        }
        if matches!(
            state.last_leg(),
            Some(Leg::Walk { .. }) | Some(Leg::Bike { .. })
        ) {
            return;
        }
        let docks = self
            .bike
            .find_nearby_stations(&state.current_location.get_coordinates(), 0.5);

        for (dock, dist_miles) in docks {
            if dock == state.current_location {
                continue;
            }
            if let Location::Station { ref id, .. } = dock {
                if !self.bike.can_unlock(id) {
                    continue;
                }
            }
            if !self.bike.can_bike_between(&state.current_location, &dock) {
                continue;
            }
            let arrival_time = state.current_time + state.current_location.walk_duration(&dock);
            self.try_step(
                state,
                dock.clone(),
                arrival_time,
                Leg::Walk {
                    from: state.current_location.clone(),
                    to: dock,
                    distance_miles: dist_miles,
                    departure_time: state.current_time,
                    arrival_time,
                },
            );
        }
    }

    fn explore_bike_rides(&mut self, state: &SearchState) {
        if !self.query.modes.allows(TransitMode::Bike) {
            return;
        }
        let candidate_docks = self
            .bike
            .find_nearby_stations(&state.current_location.get_coordinates(), 3.0);

        for (dock, dist_miles) in candidate_docks {
            if dock == state.current_location {
                continue;
            }
            if let Location::Station { ref id, .. } = dock {
                if !self.bike.can_dock(id) {
                    continue;
                }
            }
            if !self.bike.can_bike_between(&state.current_location, &dock) {
                continue;
            }
            let bike_duration = state.current_location.bike_duration(&dock) + Time::from_minutes(1);
            let arrival_time = state.current_time + bike_duration;
            self.try_step(
                state,
                dock.clone(),
                arrival_time,
                Leg::Bike {
                    from: state.current_location.clone(),
                    to: dock,
                    distance_miles: dist_miles,
                    departure_time: state.current_time,
                    arrival_time,
                },
            );
        }
    }

    fn explore_transit(&mut self, state: &SearchState) {
        let active_services = self
            .transit
            .schedule
            .active_services_for_date(&state.current_time.date);

        if let Some(edges) = self
            .transit
            .graph
            .adjacency_list
            .get(&state.current_location)
        {
            for edge in edges {
                let is_staying_on_train = match state.last_leg() {
                    Some(Leg::Transit { trip_id, .. }) => {
                        edge.departures.iter().any(|d| &d.trip_id == trip_id)
                    }
                    _ => false,
                };

                let buffer = match &state.current_location {
                    Location::Station { id, .. } => self
                        .transit
                        .stations
                        .get_station(id)
                        .map_or(Time::from_minutes(2), |s| s.boarding_buffer()),
                    _ => Time::from_minutes(2),
                };

                let min_dep_time = if is_staying_on_train {
                    state.current_time.time
                } else {
                    state.current_time.time + buffer
                };

                if let Some(departure) = edge.next_departure(min_dep_time, &active_services, &self.query.modes) {
                    let dep_datetime =
                        DateTime::new(state.current_time.date, departure.departure_time);
                    let arrival_time = dep_datetime + departure.travel_time;

                    self.try_step(
                        state,
                        edge.to.clone(),
                        arrival_time,
                        Leg::Transit {
                            from: state.current_location.clone(),
                            to: edge.to.clone(),
                            trip_id: departure.trip_id.clone(),
                            departure_time: dep_datetime,
                            arrival_time,
                            stops_count: 1,
                        },
                    );
                }
            }
        }
    }

    fn explore_walk_to_destination(&mut self, state: &SearchState) {
        if !self.bike.can_bike_between(&state.current_location, &self.query.destination) {
            return;
        }
        let arrival_time = state.current_time
            + state
                .current_location
                .walk_duration(&self.query.destination);
        self.try_step(
            state,
            self.query.destination.clone(),
            arrival_time,
            Leg::Walk {
                from: state.current_location.clone(),
                to: self.query.destination.clone(),
                distance_miles: state.current_location.walk_miles(&self.query.destination),
                departure_time: state.current_time,
                arrival_time,
            },
        );
    }
}

pub fn find_route(transit: &TransitNetwork, bike: &BikeNetwork, query: Query) -> Option<Plan> {
    let mut ctx = SearchContext::new(transit, bike, &query);

    while let Some(state) = ctx.pq.pop() {
        // 1) Destination reached
        if state.current_location == query.destination {
            let departure_time = ctx.departure_time;
            return Some(Plan {
                origin: query.origin.clone(),
                destination: query.destination.clone(),
                departure_time,
                arrival_time: state.current_time,
                legs: merge_consecutive_transit_legs(state.path),
            });
        }

        // 2) Pruning
        if let Some(&best) = ctx.best_times.get(&state.current_location)
            && state.current_time > best
        {
            continue;
        }

        // 3) Explore edges with state-based dispatch
        if ctx.is_bike_dock(&state.current_location)
            && matches!(state.last_leg(), Some(Leg::Walk { .. }))
        {
            // Just walked to a bike dock -> unlock & ride
            ctx.explore_bike_rides(&state);
        } else {
            ctx.explore_transit(&state);
            ctx.explore_walk_to_transit(&state);
            ctx.explore_walk_to_bike_docks(&state);
        }

        ctx.explore_walk_to_destination(&state);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{Coordinates, Date};

    #[test]
    fn test_plan_stats_calculation() {
        let date = Date::new(2026, 9, 21);
        let dep = DateTime::new(date, Time::from_minutes(8 * 60));

        let walk_dep = dep;
        let walk_arr = DateTime::new(date, Time::from_minutes(8 * 60 + 10));

        // 5 minute wait before transit
        let transit_dep = DateTime::new(date, Time::from_minutes(8 * 60 + 15));
        let transit_arr = DateTime::new(date, Time::from_minutes(8 * 60 + 30));

        let leg1 = Leg::Walk {
            from: Location::Point(Coordinates::new(40.0, -74.0)),
            to: Location::Point(Coordinates::new(40.01, -74.0)),
            distance_miles: 0.5,
            departure_time: walk_dep,
            arrival_time: walk_arr,
        };

        let leg2 = Leg::Transit {
            from: Location::Point(Coordinates::new(40.01, -74.0)),
            to: Location::Point(Coordinates::new(40.05, -74.0)),
            trip_id: "test_trip".to_string(),
            departure_time: transit_dep,
            arrival_time: transit_arr,
            stops_count: 3,
        };

        let plan = Plan {
            origin: Location::Point(Coordinates::new(40.0, -74.0)),
            destination: Location::Point(Coordinates::new(40.05, -74.0)),
            departure_time: dep,
            arrival_time: transit_arr,
            legs: vec![leg1, leg2],
        };

        let stats = plan.stats();
        assert_eq!(stats.total_duration_mins, 30);
        assert_eq!(stats.walk_mins, 10);
        assert_eq!(stats.transit_mins, 15);
        assert_eq!(stats.wait_mins, 5);
        assert_eq!(stats.bike_mins, 0);
        assert_eq!(stats.walk_miles, 0.5);
        assert_eq!(stats.bike_miles, 0.0);
    }
}

