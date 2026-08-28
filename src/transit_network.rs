use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    path::Path,
    str::FromStr,
};

use serde::Deserialize;

use crate::{
    graph::{Departure, Edge, Graph},
    util::{Coordinates, Date, DateTime, DayOfWeek, Location, Time, TransitMode},
};

// ============================================================================
// Real-Time Transit Feed Endpoints (Reference)
// ============================================================================
//
// 1. PATH (Port Authority of NY & NJ):
//    - Official RidePATH JSON (live countdowns):
//      https://www.panynj.gov/bin/portauthority/ridepath.json
//    - Community GTFS-RT Protobuf feed:
//      https://path.transitdata.nyc/gtfsrt
//
// 2. MTA NYC Subway (GTFS-RT Protobuf Feeds):
//    - Lines 1, 2, 3, 4, 5, 6, S:
//      https://api-endpoint.mta.info/Dataservice/mtagtfsfeeds/nyct%2Fgtfs
//    - Lines A, C, E:
//      https://api-endpoint.mta.info/Dataservice/mtagtfsfeeds/nyct%2Fgtfs-ace
//    - Lines B, D, F, M:
//      https://api-endpoint.mta.info/Dataservice/mtagtfsfeeds/nyct%2Fgtfs-bdfm
//    - Lines G:
//      https://api-endpoint.mta.info/Dataservice/mtagtfsfeeds/nyct%2Fgtfs-g
//    - Lines J, Z:
//      https://api-endpoint.mta.info/Dataservice/mtagtfsfeeds/nyct%2Fgtfs-jz
//    - Lines N, Q, R, W:
//      https://api-endpoint.mta.info/Dataservice/mtagtfsfeeds/nyct%2Fgtfs-nqrw
//    - Lines L:
//      https://api-endpoint.mta.info/Dataservice/mtagtfsfeeds/nyct%2Fgtfs-l
//    - Lines 7:
//      https://api-endpoint.mta.info/Dataservice/mtagtfsfeeds/nyct%2Fgtfs-7
//
// 3. NJ Transit / HBLR (Hudson-Bergen Light Rail):
//    - NJ Transit Developer Portal (Registration required for GTFS-RT):
//      https://developer.njtransit.com/
//    - Live DepartureVision:
//      https://dv.njtransit.com/
// ============================================================================

// ============================================================================
// Raw GTFS CSV Structs
// ============================================================================

#[derive(Debug, Deserialize)] pub struct TransitStopRaw {
    pub stop_id: String,
    pub stop_name: String,
    pub stop_lat: f64,
    pub stop_lon: f64,
    #[serde(default)]
    pub parent_station: String,
}

#[derive(Debug, Deserialize)]
pub struct CalendarRaw {
    pub service_id: String,
    #[serde(default)]
    pub monday: u8,
    #[serde(default)]
    pub tuesday: u8,
    #[serde(default)]
    pub wednesday: u8,
    #[serde(default)]
    pub thursday: u8,
    #[serde(default)]
    pub friday: u8,
    #[serde(default)]
    pub saturday: u8,
    #[serde(default)]
    pub sunday: u8,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Deserialize)]
pub struct TripRaw {
    pub trip_id: String,
    pub service_id: String,
}

// ============================================================================
// Domain Models
// ============================================================================

#[derive(Debug, Clone)]
pub struct TransitStation {
    pub id: String,
    pub name: String,
    pub coordinates: Coordinates,
}

impl TransitStation {
    pub fn to_location(&self) -> Location {
        Location::Station {
            id: self.id.clone(),
            name: self.name.clone(),
            coords: self.coordinates,
        }
    }

    pub fn boarding_buffer(&self) -> Time {
        let raw_id = self.id.strip_prefix("path:").unwrap_or(&self.id);
        match raw_id {
            "26734" => Time::from_minutes(5),
            _ => Time::from_minutes(2),
        }
    }
}

#[derive(Debug, Default)]
pub struct TransitStationDirectory {
    pub stations: HashMap<String, TransitStation>,
    pub stop_to_station: HashMap<String, String>,
}

impl TransitStationDirectory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_from_gtfs<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        agency_prefix: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(file_path)?;
        let mut raw_stops: Vec<TransitStopRaw> = Vec::new();

        for result in rdr.deserialize() {
            raw_stops.push(result?);
        }

        let prefix_id = |id: &str| -> String {
            match agency_prefix {
                Some(p) if !p.is_empty() => format!("{}:{}", p, id),
                _ => id.to_string(),
            }
        };

        for stop in &raw_stops {
            if stop.parent_station.is_empty() {
                let station_id = prefix_id(&stop.stop_id);
                self.stations.insert(
                    station_id.clone(),
                    TransitStation {
                        id: station_id.clone(),
                        name: stop.stop_name.clone(),
                        coordinates: Coordinates::new(stop.stop_lat, stop.stop_lon),
                    },
                );
                self.stop_to_station
                    .insert(stop.stop_id.clone(), station_id);
            }
        }

        for stop in &raw_stops {
            if !stop.parent_station.is_empty() {
                let parent_id = prefix_id(&stop.parent_station);
                self.stop_to_station.insert(stop.stop_id.clone(), parent_id);
            }
        }

        Ok(())
    }

    pub fn resolve_stop_to_station(&self, stop_id: &str) -> Option<&str> {
        self.stop_to_station.get(stop_id).map(|s| s.as_str())
    }

    pub fn get_station(&self, station_id: &str) -> Option<&TransitStation> {
        self.stations.get(station_id)
    }

    pub fn get_name(&self, station_id: &str) -> &str {
        self.stations
            .get(station_id)
            .map(|s| s.name.as_str())
            .unwrap_or("Unknown Station")
    }

    pub fn get_location(&self, station_id: &str) -> Option<Location> {
        self.stations.get(station_id).map(|s| s.to_location())
    }

    pub fn find_nearby_stations(
        &self,
        coords: &Coordinates,
        max_distance_miles: f64,
    ) -> Vec<(Location, f64)> {
        self.stations
            .values()
            .map(|s| {
                let location = s.to_location();
                let dist = coords.distance_to(&s.coordinates);
                (location, dist)
            })
            .filter(|(_, dist)| *dist <= max_distance_miles)
            .collect()
    }
}

impl Display for TransitStationDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut stations: Vec<_> = self.stations.values().collect();
        stations.sort_by_key(|s| &s.id);

        writeln!(f, "\n**Transit Stations**\n")?;
        for station in stations {
            writeln!(
                f,
                "station: {} ({}) at ({:.4}, {:.4})",
                station.name, station.id, station.coordinates.lat, station.coordinates.lon
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CalendarService {
    pub service_id: String,
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,
    pub start_date: Date,
    pub end_date: Date,
}

impl TryFrom<CalendarRaw> for CalendarService {
    type Error = Box<dyn Error>;

    fn try_from(raw: CalendarRaw) -> Result<Self, Self::Error> {
        let start_date = Date::from_str(&raw.start_date)?;
        let end_date = Date::from_str(&raw.end_date)?;

        Ok(Self {
            service_id: raw.service_id,
            monday: raw.monday == 1,
            tuesday: raw.tuesday == 1,
            wednesday: raw.wednesday == 1,
            thursday: raw.thursday == 1,
            friday: raw.friday == 1,
            saturday: raw.saturday == 1,
            sunday: raw.sunday == 1,
            start_date,
            end_date,
        })
    }
}

impl CalendarService {
    pub fn runs_on(&self, day_of_week: DayOfWeek) -> bool {
        match day_of_week {
            DayOfWeek::Monday => self.monday,
            DayOfWeek::Tuesday => self.tuesday,
            DayOfWeek::Wednesday => self.wednesday,
            DayOfWeek::Thursday => self.thursday,
            DayOfWeek::Friday => self.friday,
            DayOfWeek::Saturday => self.saturday,
            DayOfWeek::Sunday => self.sunday,
        }
    }

    pub fn is_active_on(&self, date: &Date) -> bool {
        if *date < self.start_date || *date > self.end_date {
            return false;
        }
        self.runs_on(date.day_of_week())
    }
}

#[derive(Debug, Deserialize)]
pub struct CalendarDateRaw {
    pub service_id: String,
    pub date: String,
    pub exception_type: u8,
}

#[derive(Debug, Default, Clone)]
pub struct Schedule {
    pub services: HashMap<String, CalendarService>,
    pub calendar_dates: HashMap<Date, HashSet<String>>,
    pub trip_to_service: HashMap<String, String>,
}

impl Schedule {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_calendar<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(), Box<dyn Error>> {
        let p = file_path.as_ref();
        if !p.exists() {
            return Ok(());
        }
        let mut rdr = csv::Reader::from_path(p)?;
        for result in rdr.deserialize() {
            let raw: CalendarRaw = result?;
            if let Ok(service) = CalendarService::try_from(raw) {
                self.services.insert(service.service_id.clone(), service);
            }
        }
        Ok(())
    }

    pub fn load_calendar_dates<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(), Box<dyn Error>> {
        let p = file_path.as_ref();
        if !p.exists() {
            return Ok(());
        }
        let mut rdr = csv::Reader::from_path(p)?;
        for result in rdr.deserialize() {
            let raw: CalendarDateRaw = result?;
            if let Ok(date) = Date::from_str(&raw.date) {
                if raw.exception_type == 1 {
                    self.calendar_dates.entry(date).or_default().insert(raw.service_id);
                }
            }
        }
        Ok(())
    }

    pub fn load_trips<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(), Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(file_path)?;
        for result in rdr.deserialize() {
            let trip: TripRaw = result?;
            self.trip_to_service.insert(trip.trip_id, trip.service_id);
        }
        Ok(())
    }

    pub fn load_from_gtfs<P1: AsRef<Path>, P2: AsRef<Path>>(
        &mut self,
        calendar_path: P1,
        trips_path: P2,
    ) -> Result<(), Box<dyn Error>> {
        self.load_calendar(calendar_path)?;
        self.load_trips(trips_path)?;
        Ok(())
    }

    pub fn active_services_for_date(&self, date: &Date) -> HashSet<String> {
        let mut active: HashSet<String> = self.services
            .values()
            .filter(|s| s.is_active_on(date))
            .map(|s| s.service_id.clone())
            .collect();
        if let Some(date_services) = self.calendar_dates.get(date) {
            active.extend(date_services.iter().cloned());
        }
        active.insert("realtime".to_string());
        active
    }

    pub fn get_service_id(&self, trip_id: &str) -> Option<&str> {
        self.trip_to_service.get(trip_id).map(|s| s.as_str())
    }
}

// ============================================================================
// Master TransitNetwork Coordinator
// ============================================================================

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

    /// Loads and merges GTFS data (stops, calendar/calendar_dates, trips, stop_times)
    pub fn load_gtfs<P: AsRef<Path>>(
        &mut self,
        dir: P,
        agency_prefix: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let dir_path = dir.as_ref();
        let stops_path = dir_path.join("stops.txt");
        let calendar_path = dir_path.join("calendar.txt");
        let calendar_dates_path = dir_path.join("calendar_dates.txt");
        let trips_path = dir_path.join("trips.txt");
        let stop_times_path = dir_path.join("stop_times.txt");

        self.stations.load_from_gtfs(stops_path, agency_prefix)?;
        if calendar_path.exists() {
            self.schedule.load_calendar(calendar_path)?;
        }
        if calendar_dates_path.exists() {
            self.schedule.load_calendar_dates(calendar_dates_path)?;
        }
        self.schedule.load_trips(trips_path)?;

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

    /// Find nearby transit stations within `max_distance_miles` of a coordinate
    pub fn find_nearby_stations(
        &self,
        coords: &Coordinates,
        max_distance_miles: f64,
    ) -> Vec<(Location, f64)> {
        self.stations.find_nearby_stations(coords, max_distance_miles)
    }

    /// Overlays live real-time PATH departures onto the transit graph
    pub fn refresh_realtime(&mut self, current_time: &DateTime) -> Result<usize, Box<dyn Error>> {
        let live_departures = crate::realtime::fetch_path_realtime()?;
        let count = live_departures.len();

        for live in live_departures {
            let from_loc = match self
                .stations
                .stations
                .values()
                .find(|s| s.name.to_lowercase().contains(&live.station_name.to_lowercase()))
            {
                Some(s) => s.to_location(),
                None => continue,
            };

            let dest_name = match live.destination_code.to_uppercase().as_str() {
                "WTC" => "World Trade Center",
                "33S" | "33RD" => "33rd Street",
                "HOB" => "Hoboken",
                "JSQ" => "Journal Square",
                "NWK" => "Newark",
                "EXP" => "Exchange Place",
                "NEW" => "Newport",
                "HAR" => "Harrison",
                "GRV" => "Grove Street",
                "CHR" => "Christopher Street",
                "14S" => "14th Street",
                "23S" => "23rd Street",
                "9ST" => "9th Street",
                _ => continue,
            };

            let to_loc = match self
                .stations
                .stations
                .values()
                .find(|s| s.name.to_lowercase().contains(&dest_name.to_lowercase()))
            {
                Some(s) => s.to_location(),
                None => continue,
            };

            let dep_mins = current_time.time.as_minutes() + (live.seconds_to_arrival as u32 / 60);
            let dep_time = Time::from_minutes(dep_mins);

            let edges = self.graph.adjacency_list.entry(from_loc.clone()).or_default();
            let travel_time = edges
                .iter()
                .find(|e| e.to == to_loc)
                .and_then(|e| e.departures.first().map(|d| d.travel_time))
                .unwrap_or(Time::from_minutes(5));

            let departure = Departure {
                trip_id: format!("live_{}_{}", live.station_code, live.destination_code),
                service_id: "realtime".to_string(),
                mode: TransitMode::Path,
                departure_time: dep_time,
                travel_time,
            };

            if let Some(edge) = edges.iter_mut().find(|e| e.to == to_loc) {
                edge.departures.push(departure);
                edge.departures.sort_by_key(|d| d.departure_time);
            } else {
                edges.push(Edge {
                    to: to_loc,
                    departures: vec![departure],
                });
            }
        }

        Ok(count)
    }
}
