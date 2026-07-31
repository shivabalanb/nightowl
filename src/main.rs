use std::error::Error;

use nightowl::ingestor::{load_transit_stop_times, load_transit_stops};

fn main() -> Result<(), Box<dyn Error>> {
    let stops = load_transit_stops("data/path/stops.txt")?;

    println!("loaded {} stations!\n", stops.len());

    for stop in stops.iter().take(10) {
        println!(
            "station: {} ({}) at ({:.4}, {:.4})",
            stop.stop_name, stop.stop_id, stop.stop_lat, stop.stop_lon
        );
    }

    let stop_times = load_transit_stop_times("data/path/stop_times.txt")?;

    println!("\nloaded {} scheduled stop_times!\n", stop_times.len());

    Ok(())
}
