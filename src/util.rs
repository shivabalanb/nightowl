use std::{
    error::Error,
    fmt::Display,
    ops::{Add, Sub},
    str::FromStr,
};

const EARTH_RADIUS_MILES: f64 = 3959.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
                if is_leap {
                    29
                } else {
                    28
                }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime {
    pub date: Date,
    pub time: Time,
}

impl DateTime {
    pub fn new(date: Date, time: Time) -> Self {
        Self { date, time }
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
        write!(f, "{} {} ({})", self.date, self.time, self.date.day_of_week())
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
        let minutes = (dist_miles * 24.0).round() as u32;
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
