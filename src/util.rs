use std::{
    error::Error,
    fmt::Display,
    ops::{Add, Sub},
    str::FromStr,
};

const EARTH_RADIUS_MILES: f64 = 3959.0;
const WALKING_SPEED: f64 = 22.0; // 22 mins/mile
const BIKING_SPEED: f64 = 7.0; // 7 mins/mile

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Display for DayOfWeek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DayOfWeek::Monday => write!(f, "Monday"),
            DayOfWeek::Tuesday => write!(f, "Tuesday"),
            DayOfWeek::Wednesday => write!(f, "Wednesday"),
            DayOfWeek::Thursday => write!(f, "Thursday"),
            DayOfWeek::Friday => write!(f, "Friday"),
            DayOfWeek::Saturday => write!(f, "Saturday"),
            DayOfWeek::Sunday => write!(f, "Sunday"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct Date {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    pub fn new(year: u32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Calculates day of the week using Sakamoto's algorithm
    pub fn day_of_week(&self) -> DayOfWeek {
        let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let mut y = self.year;
        if self.month < 3 {
            y -= 1;
        }
        let d = (y + y / 4 - y / 100 + y / 400 + t[(self.month - 1) as usize] + self.day) % 7;
        match d {
            0 => DayOfWeek::Sunday,
            1 => DayOfWeek::Monday,
            2 => DayOfWeek::Tuesday,
            3 => DayOfWeek::Wednesday,
            4 => DayOfWeek::Thursday,
            5 => DayOfWeek::Friday,
            6 => DayOfWeek::Saturday,
            _ => unreachable!(),
        }
    }

    /// Next calendar day
    pub fn next_day(&self) -> Self {
        let days_in_month = match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                let is_leap =
                    (self.year % 4 == 0 && self.year % 100 != 0) || (self.year % 400 == 0);
                if is_leap { 29 } else { 28 }
            }
            _ => 30,
        };

        if self.day < days_in_month {
            Date {
                year: self.year,
                month: self.month,
                day: self.day + 1,
            }
        } else if self.month < 12 {
            Date {
                year: self.year,
                month: self.month + 1,
                day: 1,
            }
        } else {
            Date {
                year: self.year + 1,
                month: 1,
                day: 1,
            }
        }
    }
}

impl FromStr for Date {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean = s.trim();
        if clean.len() == 8 && clean.chars().all(|c| c.is_ascii_digit()) {
            let year: u32 = clean[0..4].parse()?;
            let month: u32 = clean[4..6].parse()?;
            let day: u32 = clean[6..8].parse()?;
            Ok(Date { year, month, day })
        } else if clean.contains('-') {
            let mut parts = clean.split('-');
            let year: u32 = parts.next().ok_or("missing year")?.parse()?;
            let month: u32 = parts.next().ok_or("missing month")?.parse()?;
            let day: u32 = parts.next().ok_or("missing day")?.parse()?;
            Ok(Date { year, month, day })
        } else {
            Err(format!("invalid date format: {}", s).into())
        }
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct Time(pub u32);

impl Time {
    pub const MAX: Self = Self(u32::MAX);

    pub fn from_minutes(minutes: u32) -> Self {
        Self(minutes)
    }
    pub fn as_minutes(&self) -> u32 {
        self.0
    }
    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
    pub fn duration_to(&self, later_time: Time) -> u32 {
        later_time.as_minutes().saturating_sub(self.as_minutes())
    }
}

impl Add for Time {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Time(self.0 + rhs.0)
    }
}

impl Sub for Time {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Time(self.0 - rhs.0)
    }
}

impl FromStr for Time {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');
        let h_str = parts.next().ok_or("missing hours")?;
        let m_str = parts.next().ok_or("missing minutes")?;

        let hours: u32 = h_str.trim().parse()?;
        let minutes: u32 = m_str.trim().parse()?;

        Ok(Time(hours * 60 + minutes))
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hours = (self.0 / 60) % 24;
        let mins = self.0 % 60;
        write!(f, "{:02}:{:02} EST", hours, mins)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct DateTime {
    pub date: Date,
    pub time: Time,
}

impl Default for DateTime {
    fn default() -> Self {
        Self::now()
    }
}

impl DateTime {
    pub fn new(date: Date, time: Time) -> Self {
        Self { date, time }
    }

    pub fn now() -> Self {
        use chrono::{Datelike, Timelike};
        let utc = chrono::Utc::now();
        let year = utc.year();

        // US Eastern Time Daylight Saving Time:
        // Starts 2nd Sunday of March at 2:00 AM EST (07:00 UTC)
        // Ends 1st Sunday of November at 2:00 AM EDT (06:00 UTC)
        let march_1_weekday = chrono::NaiveDate::from_ymd_opt(year, 3, 1)
            .unwrap()
            .weekday()
            .num_days_from_sunday();
        let first_sun_mar = 1 + (7 - march_1_weekday) % 7;
        let second_sun_mar = first_sun_mar + 7;
        let dst_start_utc = chrono::NaiveDate::from_ymd_opt(year, 3, second_sun_mar)
            .unwrap()
            .and_hms_opt(7, 0, 0)
            .unwrap();

        let nov_1_weekday = chrono::NaiveDate::from_ymd_opt(year, 11, 1)
            .unwrap()
            .weekday()
            .num_days_from_sunday();
        let first_sun_nov = 1 + (7 - nov_1_weekday) % 7;
        let dst_end_utc = chrono::NaiveDate::from_ymd_opt(year, 11, first_sun_nov)
            .unwrap()
            .and_hms_opt(6, 0, 0)
            .unwrap();

        let naive_utc = utc.naive_utc();
        let offset_hours = if naive_utc >= dst_start_utc && naive_utc < dst_end_utc {
            -4 // EDT
        } else {
            -5 // EST
        };

        let eastern = naive_utc + chrono::Duration::hours(offset_hours);
        DateTime {
            date: Date::new(eastern.year() as u32, eastern.month(), eastern.day()),
            time: Time::from_minutes(eastern.hour() * 60 + eastern.minute()),
        }
    }

    pub fn add_minutes(&self, minutes: u32) -> Self {
        let total_mins = self.time.as_minutes() + minutes;
        let days_to_add = total_mins / 1440;
        let rem_mins = total_mins % 1440;

        let mut date = self.date;
        for _ in 0..days_to_add {
            date = date.next_day();
        }

        DateTime {
            date,
            time: Time::from_minutes(rem_mins),
        }
    }
}

impl Add<Time> for DateTime {
    type Output = Self;
    fn add(self, rhs: Time) -> Self {
        self.add_minutes(rhs.as_minutes())
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} ({})",
            self.date,
            self.time,
            self.date.day_of_week()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lon: f64,
}

impl Eq for Coordinates {}

impl std::hash::Hash for Coordinates {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.lat.to_bits().hash(state);
        self.lon.to_bits().hash(state);
    }
}

impl Coordinates {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
    fn planar_offset_miles(&self, other: &Coordinates) -> (f64, f64) {
        let delta_lat = self.lat.to_radians() - other.lat.to_radians();
        let mean_lat = (self.lat.to_radians() + other.lat.to_radians()) / 2.0;
        let delta_lon = self.lon.to_radians() - other.lon.to_radians();

        let x = (delta_lon * mean_lat.cos() * EARTH_RADIUS_MILES).abs();
        let y = (delta_lat * EARTH_RADIUS_MILES).abs();
        (x, y)
    }

    pub fn distance_to(&self, other: &Coordinates) -> f64 {
        let (x, y) = self.planar_offset_miles(other);
        (x * x + y * y).sqrt()
    }

    pub fn manhattan_distance_to(&self, other: &Coordinates) -> f64 {
        let (x, y) = self.planar_offset_miles(other);
        x + y
    }

    /// Resolves an address string, common alias, or "lat, lon" pair into geographical Coordinates.
    pub fn parse_or_geocode(input: &str) -> Result<Coordinates, Box<dyn Error>> {
        let clean = input.trim();

        // 1. Try raw "lat, lon"
        if let Some((lat_str, lon_str)) = clean.split_once(',') {
            if let (Ok(lat), Ok(lon)) = (lat_str.trim().parse::<f64>(), lon_str.trim().parse::<f64>()) {
                return Ok(Coordinates::new(lat, lon));
            }
        }

        // 2. Built-in NYC/NJ Address Book
        let lower = clean.to_lowercase();
        match lower.as_str() {
            "home" | "apartment" | "my place" => {
                return Ok(Coordinates::new(40.72204775835277, -74.0368774356056));
            }
            "work" | "office" => {
                return Ok(Coordinates::new(40.749719, -73.987823));
            }
            "vital" | "vital les" | "vital climbing" | "vital gym" => {
                return Ok(Coordinates::new(40.71721004390394, -73.98642630279129));
            }
            "vital brooklyn" | "vital williamsburg" | "vital bk" => {
                return Ok(Coordinates::new(40.721867, -73.958742));
            }
            "williamsburg" => {
                return Ok(Coordinates::new(40.7163, -73.9586));
            }
            "dumbo" => {
                return Ok(Coordinates::new(40.7033, -73.9892));
            }
            "wtc" | "world trade center" | "oculus" => {
                return Ok(Coordinates::new(40.712582, -74.009781));
            }
            "herald sq" | "herald square" | "34th st" | "midtown" => {
                return Ok(Coordinates::new(40.749719, -73.987823));
            }
            "times sq" | "times square" => {
                return Ok(Coordinates::new(40.758896, -73.985130));
            }
            "grove st" | "grove street" | "grove" => {
                return Ok(Coordinates::new(40.7196, -74.0431));
            }
            "newport" | "newport path" => {
                return Ok(Coordinates::new(40.7270, -74.0346));
            }
            "exchange place" | "exchange pl" | "exchange" => {
                return Ok(Coordinates::new(40.71676, -74.03238));
            }
            "hoboken" | "hoboken terminal" => {
                return Ok(Coordinates::new(40.7350, -74.0290));
            }
            "journal square" | "jsq" => {
                return Ok(Coordinates::new(40.7320, -74.0628));
            }
            "union square" | "union sq" => {
                return Ok(Coordinates::new(40.7359, -73.9911));
            }
            "grand central" => {
                return Ok(Coordinates::new(40.7527, -73.9772));
            }
            _ => {}
        }

        // 3. Fallback to OpenStreetMap Nominatim Geocoding API bounded to NY/NJ metro
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(4))
            .user_agent("NightowlTransitRouter/1.0")
            .build()?;

        #[derive(serde::Deserialize)]
        struct NominatimResult {
            lat: String,
            lon: String,
        }

        let url = format!(
            "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=1&viewbox=-74.3,40.9,-73.7,40.5",
            clean.replace(' ', "+")
        );

        let response = match client.get(&url).send() {
            Ok(resp) => resp,
            Err(_) => {
                return Err(format!(
                    "Address '{}' not recognized offline and live geocoding service is unavailable.",
                    input
                )
                .into());
            }
        };

        if let Ok(res) = response.json::<Vec<NominatimResult>>() {
            if let Some(first) = res.first() {
                if let (Ok(lat), Ok(lon)) = (first.lat.parse::<f64>(), first.lon.parse::<f64>()) {
                    return Ok(Coordinates::new(lat, lon));
                }
            }
        }

        Err(format!(
            "Address '{}' was not found in the NYC/NJ metro area.",
            input
        )
        .into())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Location {
    Station {
        id: String,
        name: String,
        coords: Coordinates,
    },
    Point(Coordinates),
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Location::Station { id: id1, .. }, Location::Station { id: id2, .. }) => id1 == id2,
            (Location::Point(c1), Location::Point(c2)) => c1 == c2,
            _ => false,
        }
    }
}

impl Eq for Location {}

impl std::hash::Hash for Location {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Location::Station { id, .. } => {
                0u8.hash(state);
                id.hash(state);
            }
            Location::Point(coords) => {
                1u8.hash(state);
                coords.hash(state);
            }
        }
    }
}

impl Location {
    pub fn get_coordinates(&self) -> Coordinates {
        match self {
            Location::Point(coords) => *coords,
            Location::Station { coords, .. } => *coords,
        }
    }

    pub fn walk_duration(&self, other: &Location) -> Time {
        let dist_miles = self.walk_miles(other);
        let minutes = (dist_miles * WALKING_SPEED).round() as u32;
        Time::from_minutes(minutes)
    }

    pub fn walk_miles(&self, other: &Location) -> f64 {
        self.get_coordinates()
            .manhattan_distance_to(&other.get_coordinates())
    }

    pub fn bike_duration(&self, other: &Location) -> Time {
        let dist_miles = self.bike_miles(other);
        let minutes = (dist_miles * BIKING_SPEED).round() as u32;
        Time::from_minutes(minutes)
    }

    pub fn bike_miles(&self, other: &Location) -> f64 {
        self.get_coordinates()
            .manhattan_distance_to(&other.get_coordinates())
    }

    pub fn is_bike_dock(&self) -> bool {
        matches!(self, Location::Station { id, .. } if id.starts_with("citibike:"))
    }

    pub fn is_transit_station(&self) -> bool {
        matches!(self, Location::Station { id, .. } if !id.starts_with("citibike:"))
    }

    pub fn name(&self) -> String {
        match self {
            Location::Station { id, name, .. } => {
                if id.starts_with("citibike:") {
                    format!("{} (Citi Bike)", name)
                } else if id.starts_with("path:") {
                    format!("{} (PATH)", name)
                } else if id.starts_with("hblr:") {
                    format!("{} (HBLR)", name)
                } else if id.starts_with("mta:") {
                    format!("{} (MTA Subway)", name)
                } else {
                    name.clone()
                }
            }
            Location::Point(coords) => format!("Point ({:.4}, {:.4})", coords.lat, coords.lon),
        }
    }
}

// ============================================================================
// Transportation Modes & Filter Sets
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitMode {
    Walk,
    Bike,
    Path,
    Mta,
    Hblr,
}

impl std::fmt::Display for TransitMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransitMode::Walk => write!(f, "walk"),
            TransitMode::Bike => write!(f, "bike"),
            TransitMode::Path => write!(f, "path"),
            TransitMode::Mta => write!(f, "mta"),
            TransitMode::Hblr => write!(f, "hblr"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModeSet {
    allowed: std::collections::HashSet<TransitMode>,
}

impl Default for ModeSet {
    fn default() -> Self {
        Self::all()
    }
}

impl ModeSet {
    pub fn all() -> Self {
        let mut allowed = std::collections::HashSet::new();
        allowed.insert(TransitMode::Walk);
        allowed.insert(TransitMode::Bike);
        allowed.insert(TransitMode::Path);
        allowed.insert(TransitMode::Mta);
        allowed.insert(TransitMode::Hblr);
        Self { allowed }
    }

    pub fn from_modes(modes: &[TransitMode]) -> Self {
        let mut allowed = std::collections::HashSet::new();
        for m in modes {
            allowed.insert(*m);
        }
        allowed.insert(TransitMode::Walk);
        Self { allowed }
    }

    pub fn allows(&self, mode: TransitMode) -> bool {
        self.allowed.contains(&mode)
    }

    pub fn enable(&mut self, mode: TransitMode) {
        self.allowed.insert(mode);
    }

    pub fn disable(&mut self, mode: TransitMode) {
        self.allowed.remove(&mode);
    }

    pub fn active_modes_string(&self) -> String {
        let mut list = Vec::new();
        for mode in &[TransitMode::Walk, TransitMode::Bike, TransitMode::Path, TransitMode::Mta, TransitMode::Hblr] {
            if self.allows(*mode) {
                list.push(mode.to_string());
            }
        }
        list.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datetime_now_eastern_time() {
        let now = DateTime::now();
        assert!(now.date.year >= 2024);
        assert!(now.date.month >= 1 && now.date.month <= 12);
        assert!(now.date.day >= 1 && now.date.day <= 31);
        assert!(now.time.as_minutes() < 1440);
    }
}

