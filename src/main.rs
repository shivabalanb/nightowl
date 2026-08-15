use std::error::Error;

use nightowl::{
    graph::Graph,
    ingestor::{
        StationDirectory, extract_transit_segments, group_stop_times_by_trip,
        load_transit_stations, load_transit_stop_times,
    },
    router::{Query, find_route},
    util::{Coordinates, Location, Time},
};

fn main() -> Result<(), Box<dyn Error>> {
    let stations = load_transit_stations("data/path/stops.txt")?;
    let station_dir = StationDirectory::new(stations);

    let stop_times = load_transit_stop_times("data/path/stop_times.txt", station_dir.parent_map())?;

    let grouped_trips = group_stop_times_by_trip(stop_times);

    let segments = extract_transit_segments(&grouped_trips);

    println!("\n**BUILD GRAPH**\n");
    let transit_graph = Graph::from_segments(segments, &station_dir);

    println!(
        "Built graph with {} origin stations!",
        transit_graph.adjacency_list.len()
    );

    println!("\n**FIND ROUTE**\n");

    let departure_time: Time = "10:22".parse()?;

    let query = Query {
        origin: Location::Point(Coordinates::new(40.730009, -74.034637)), // Your location
        destination: Location::Point(Coordinates::new(40.7405842, -73.9858367)), // 315 Park Ave S
        departure_time,
    };

    match find_route(&transit_graph, &station_dir, query) {
        Some(plan) => {
            println!("{}", plan);
        }
        None => {
            println!("\n No route found.");
        }
    }

    Ok(())
}
