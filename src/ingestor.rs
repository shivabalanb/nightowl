use std::{collections::HashMap, error::Error, path::Path};

use crate::model::{TransitSegment, TransitSegmentDetail, TransitStop, TransitStopTime};

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

pub fn group_stop_times_by_trip(
    stop_times: Vec<TransitStopTime>,
) -> HashMap<String, Vec<TransitStopTime>> {
    let mut trips: HashMap<String, Vec<TransitStopTime>> = HashMap::new();

    for stop_time in stop_times {
        trips
            .entry(stop_time.trip_id.clone())
            .or_default()
            .push(stop_time);
    }

    for stops in trips.values_mut() {
        stops.sort_by_key(|s| s.stop_sequence);
    }

    trips
}

pub fn extract_transit_segments(
    grouped_stop_times: &HashMap<String, Vec<TransitStopTime>>,
) -> Vec<TransitSegment> {
    let mut transit_connections: Vec<TransitSegment> = Vec::new();
    for (trip_id, stop_times) in grouped_stop_times {
        for pair in stop_times.windows(2) {
            let from_stop = &pair[0];
            let to_stop = &pair[1];

            if let (Ok(b_arr_time), Ok(a_dep_time)) = (
                parse_time(&to_stop.arrival_time),
                parse_time(&from_stop.departure_time),
            ) {
                let travel_time = b_arr_time.saturating_sub(a_dep_time);
                let transit_segment_detail = TransitSegmentDetail {
                    trip_id: trip_id.clone(),
                    departure_time: a_dep_time,
                    travel_time,
                };
                let transit_segment = TransitSegment {
                    from_stop_id: from_stop.stop_id.clone(),
                    to_stop_id: to_stop.stop_id.clone(),
                    transit_segment_detail,
                };
                transit_connections.push(transit_segment);
            };
        }
    }
    transit_connections
}

pub fn parse_time(time_str: &str) -> Result<u32, String> {
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

