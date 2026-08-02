use std::{
    error::Error,
    fmt::Display,
    ops::{Add, Sub},
    str::FromStr,
};

const EARTH_RADIUS_MILES: f64 = 3959.0;

pub fn equirectangular_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let lat1_rad = lat1.to_radians();
    let lon1_rad = lon1.to_radians();
    let lat2_rad = lat2.to_radians();
    let lon2_rad = lon2.to_radians();

    let delta_lat = lat1_rad - lat2_rad;
    let mean_lat = (lat1_rad + lat2_rad) / 2.0;
    let delta_lon = lon2_rad - lon1_rad;

    let x = delta_lon * mean_lat.cos() * EARTH_RADIUS_MILES;
    let y = delta_lat * EARTH_RADIUS_MILES;

    (x * x + y * y).sqrt()
}

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
