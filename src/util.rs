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

impl Coordinates {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
    // equirectanguar distance
    pub fn distance_to(&self, other: &Coordinates) -> f64 {
        let delta_lat = self.lat.to_radians() - other.lat.to_radians();
        let mean_lat = (self.lat.to_radians() + other.lat.to_radians()) / 2.0;
        let delta_lon = self.lon.to_radians() - other.lon.to_radians();

        let x = delta_lon * mean_lat.cos() * EARTH_RADIUS_MILES;
        let y = delta_lat * EARTH_RADIUS_MILES;

        (x * x + y * y).sqrt()
    }
}
