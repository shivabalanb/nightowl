
#[derive(Debug, Clone)]
pub struct Landmark {
    pub name: &'static str,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone)]
pub struct TrainDeparture {
    pub time_of_day: u32, // minutes since midnight
    pub travel_time_miles: f64,
}

#[derive(Debug, Clone)]
pub enum EdgeType {
    Walk(f64),
    Transit(Vec<TrainDeparture>),
}

impl EdgeType {
    pub fn min_weight(&self) -> f64 {
        match self {
            EdgeType::Walk(dist) => *dist,
            EdgeType::Transit(departures) => departures
                .iter()
                .map(|d| d.travel_time_miles)
                .fold(f64::INFINITY, f64::min),
        }
    }
    pub fn weight_at_time(&self, current_time: u32) -> f64 {
        match self {
            EdgeType::Walk(dist_miles) => {
                // 3 mph walking speed = 1 mile / 20 mins
                dist_miles * 20.0
            }
            EdgeType::Transit(departures) => {
                let next_train = departures
                    .iter()
                    .filter(|d| d.time_of_day >= current_time)
                    .min_by_key(|d| d.time_of_day);
                match next_train {
                    Some(train) => {
                        let wait_time = (train.time_of_day - current_time) as f64;
                        // assume PATH trains travel ~25mph
                        let ride_time = train.travel_time_miles / (25.0 / 60.0);

                        wait_time + ride_time
                    }
                    None => f64::INFINITY,
                }
            }
        }
    }
}
