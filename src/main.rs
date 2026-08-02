use std::error::Error;

use nightowl::{
    ingestor::{
        extract_transit_segments, group_stop_times_by_trip, load_transit_stop_times,
        load_transit_stops, parse_time,
    },
    model::{StopDirectory, TransitGraph},
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n**LOAD TRANSIT STOPS**\n");
    let stops = load_transit_stops("data/path/stops.txt")?;

    println!("loaded {} stops!", stops.len());

    for stop in stops.iter().take(10) {
        println!(
            "stop: {} ({}) at ({:.4}, {:.4})",
            stop.stop_name, stop.stop_id, stop.stop_lat, stop.stop_lon
        );
    }

    let stop_dir = StopDirectory::new(stops);

    println!("\n**LOAD TRANSIT STOP TIMES**\n");
    let stop_times = load_transit_stop_times("data/path/stop_times.txt")?;

    println!("loaded {} scheduled stop_times!", stop_times.len());

    println!("\nGROUP STOP TIMES BY TRIP\n");
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

    println!("\n**EXTRACT TRANSIT SEGMENTS**\n");
    let segments = extract_transit_segments(&grouped_trips);
    println!("Extracted {} total transit segments!", segments.len());

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

    println!("\n**BUILD TRANSIT GRAPH**\n");
    let transit_graph = TransitGraph::from_segments(segments);

    println!(
        "Built TransitGraph with {} origin stops!",
        transit_graph.adjacency_list.len()
    );

    if let Some((from_id, edges)) = transit_graph.adjacency_list.iter().next() {
        let origin_name = stop_dir.get_name(from_id);
        println!(
            "\nStation: {} ({}) has {} outgoing connections:",
            origin_name,
            from_id,
            edges.len()
        );

        for edge in edges {
            let dest_name = stop_dir.get_name(&edge.to_stop_id);
            println!(
                "  -> Connection to {} ({}) with {} scheduled departures",
                dest_name,
                edge.to_stop_id,
                edge.departures.len()
            );
        }
    };
    Ok(())
}
