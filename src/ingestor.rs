use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    path::Path,
    str::FromStr,
};

use serde::Deserialize;

use crate::util::{Coordinates, Date, DayOfWeek, Location, Time};

#[derive(Debug, Deserialize)]
pub struct StopRaw {
    pub stop_id: String,
    pub stop_name: String,
    pub stop_lat: f64,
    pub stop_lon: f64,
    #[serde(default)]
    pub parent_station: String,
}

#[derive(Debug, Clone)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub coordinates: Coordinates,
}

impl Station {
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
pub struct StationDirectory {
    stations: HashMap<String, Station>,
    stop_to_station: HashMap<String, String>,
}

impl StationDirectory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_from_gtfs<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        agency_prefix: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(file_path)?;
        let mut raw_stops: Vec<StopRaw> = Vec::new();

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
                    Station {
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
                self.stop_to_station
                    .insert(stop.stop_id.clone(), parent_id);
            }
        }

        Ok(())
    }

    pub fn resolve_stop_to_station(&self, stop_id: &str) -> Option<&str> {
        self.stop_to_station.get(stop_id).map(|s| s.as_str())
    }

    pub fn get_station(&self, station_id: &str) -> Option<&Station> {
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

impl Display for StationDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n**LOAD TRANSIT STATIONS**\n")?;
        let mut stations: Vec<_> = self.stations.values().collect();
        stations.sort_by_key(|s| &s.id);

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

#[derive(Debug, Default, Clone)]
pub struct Calendar {
    pub services: HashMap<String, CalendarService>,
}

impl Calendar {
    pub fn load_from_gtfs<P: AsRef<Path>>(file_path: P) -> Result<Self, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(file_path)?;
        let mut services = HashMap::new();

        for result in rdr.deserialize() {
            let raw: CalendarRaw = result?;
            if let (Ok(start_date), Ok(end_date)) = (
                Date::from_str(&raw.start_date),
                Date::from_str(&raw.end_date),
            ) {
                services.insert(
                    raw.service_id.clone(),
                    CalendarService {
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
                    },
                );
            }
        }
        Ok(Calendar { services })
    }

    pub fn active_services_for_date(&self, date: &Date) -> HashSet<String> {
        self.services
            .values()
            .filter(|s| s.is_active_on(date))
            .map(|s| s.service_id.clone())
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct TripRaw {
    pub trip_id: String,
    pub service_id: String,
}

pub fn load_trip_services<P: AsRef<Path>>(
    file_path: P,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut trip_to_service = HashMap::new();
    for result in rdr.deserialize() {
        let trip: TripRaw = result?;
        trip_to_service.insert(trip.trip_id, trip.service_id);
    }
    Ok(trip_to_service)
}
