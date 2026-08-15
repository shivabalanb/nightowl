use std::{
    error::Error,
    fmt::Display,
    ops::{Add, Sub},
    str::FromStr,
};

const EARTH_RADIUS_MILES: f64 = 3959.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
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
    // East-West (x) and North-South (y) distances in miles via equirectangular projection
    fn planar_offset_miles(&self, other: &Coordinates) -> (f64, f64) {
        let delta_lat = self.lat.to_radians() - other.lat.to_radians();
        let mean_lat = (self.lat.to_radians() + other.lat.to_radians()) / 2.0;
        let delta_lon = self.lon.to_radians() - other.lon.to_radians();

        let x = (delta_lon * mean_lat.cos() * EARTH_RADIUS_MILES).abs();
        let y = (delta_lat * EARTH_RADIUS_MILES).abs();
        (x, y)
    }

    // Euclidean distance
    pub fn distance_to(&self, other: &Coordinates) -> f64 {
        let (x, y) = self.planar_offset_miles(other);
        (x * x + y * y).sqrt()
    }

    // Manhattan grid distance
    pub fn manhattan_distance_to(&self, other: &Coordinates) -> f64 {
        let (x, y) = self.planar_offset_miles(other);
        x + y
    }
}

#[derive(Debug, Clone)]
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
        let minutes = (dist_miles * 24.0).round() as u32; // 2.5mph (24 mins/mile, accounts for traffic lights)
        Time::from_minutes(minutes)
    }

    pub fn walk_miles(&self, other: &Location) -> f64 {
        self.get_coordinates()
            .manhattan_distance_to(&other.get_coordinates())
    }

    pub fn name(&self) -> String {
        match self {
            Location::Station { name, .. } => name.clone(),
            Location::Point(coords) => format!("Point ({:.4}, {:.4})", coords.lat, coords.lon),
        }
    }
}
