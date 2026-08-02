use std::error::Error;

use nightowl::{
    graph::TransitGraph,
    ingestor::{
        StopDirectory, extract_transit_segments, group_stop_times_by_trip, load_transit_stop_times,
        load_transit_stops,
    },
    router::find_route,
    util::Time,
};

fn main() -> Result<(), Box<dyn Error>> {
    let stops = load_transit_stops("data/path/stops.txt")?;
    let stop_dir = StopDirectory::new(stops);

    print!("{}", stop_dir);

    let stop_times = load_transit_stop_times("data/path/stop_times.txt", stop_dir.parent_map())?;

    let grouped_trips = group_stop_times_by_trip(stop_times);

    let segments = extract_transit_segments(&grouped_trips);

    println!("\n**BUILD GRAPH**\n");
    let transit_graph = TransitGraph::from_segments(segments);

    println!(
        "Built graph with {} origin stops!",
        transit_graph.adjacency_list.len()
    );

    println!("\n**FIND ROUTE**\n");

    let origin_id = "26732"; // Newport (26732) Hoboken (26730) Exchange Place (26727) 
    let destination_id = "26723"; // 23rd Street (26723)
    let departure_time: Time = "10:15".parse()?;

    let origin_name = stop_dir.get_name(origin_id);
    let dest_name = stop_dir.get_name(destination_id);

    println!(
        "Finding best route from {} ({}) to {} ({}) departing at {}...",
        origin_name, origin_id, dest_name, destination_id, departure_time
    );

    match find_route(&transit_graph, origin_id, destination_id, departure_time) {
        Some(plan) => {
            println!("\n{}", plan.display(&stop_dir));
        }
        None => {
            println!("\n No route found from {} to {}.", origin_name, dest_name);
        }
    }

    Ok(())
}
