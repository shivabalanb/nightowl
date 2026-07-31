use std::{error::Error, path::Path};

use crate::model::{TransitStop, TransitStopTime};

pub fn load_transit_stops<P: AsRef<Path>>(
    file_path: P,
) -> Result<Vec<TransitStop>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut stops = Vec::with_capacity(100);

    for result in rdr.deserialize() {
        let stop: TransitStop = result?;
        stops.push(stop);
    }
    Ok(stops)
}

pub fn load_transit_stop_times<P: AsRef<Path>>(
    file_path: P,
) -> Result<Vec<TransitStopTime>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut stop_times = Vec::with_capacity(160_000);

    for result in rdr.deserialize() {
        let stop_time: TransitStopTime = result?;
        stop_times.push(stop_time);
    }
    Ok(stop_times)
}

fn parse_time(time_str: &str) -> Result<u32, String> {
    let mut time = time_str.split(':');

    let hours: u32 = time
        .next()
        .expect("missing hours")
        .parse()
        .expect("invalid hours");

    let minutes: u32 = time
        .next()
        .expect("missing minutes")
        .parse()
        .expect("invalid minutes");
    Ok(hours * 60 + minutes)
}
