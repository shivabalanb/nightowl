use std::error::Error;

use nightowl::{
    ingestor::{
        extract_transit_segments, group_stop_times_by_trip, load_transit_stop_times,
        load_transit_stops, parse_time,
    },
    model::StopDirectory,
};

fn main() -> Result<(), Box<dyn Error>> {
    let stops = load_transit_stops("data/path/stops.txt")?;

    println!("loaded {} stops!\n", stops.len());

    for stop in stops.iter().take(10) {
        println!(
            "stop: {} ({}) at ({:.4}, {:.4})",
            stop.stop_name, stop.stop_id, stop.stop_lat, stop.stop_lon
        );
    }

    let stop_dir = StopDirectory::new(stops);

    let stop_times = load_transit_stop_times("data/path/stop_times.txt")?;

    println!("\nloaded {} scheduled stop_times!\n", stop_times.len());

    let grouped_trips = group_stop_times_by_trip(stop_times);
    println!("Grouped into {} unique trips!", grouped_trips.len());

    if let Some((trip_id, stops)) = grouped_trips.iter().next() {
        println!("\nTrip ID: {}", trip_id);
        for stop in stops {
            println!(
                "  Stop {} -> Stop {}, departure_time {:?}",
                stop.stop_sequence,
                stop_dir.get_name(&stop.stop_id),
                parse_time(&stop.departure_time)
            );
        }
    }

    let segments = extract_transit_segments(&grouped_trips);
    println!("\nExtracted {} total transit segments!", segments.len());

    if let Some(first_segment) = segments.first() {
        println!(
            "\nSample connection: Stop {} -> Stop {} (Trip: {}, Departs: {}m, Duration: {}m)",
            stop_dir.get_name(&first_segment.from_stop_id),
            stop_dir.get_name(&first_segment.to_stop_id),
            first_segment.transit_segment_detail.trip_id,
            first_segment.transit_segment_detail.departure_time,
            first_segment.transit_segment_detail.travel_time,
        );
    }
    Ok(())
}
