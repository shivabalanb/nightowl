use std::error::Error;

use nightowl::{
    graph::Graph,
    ingestor::StationDirectory,
    router::{Query, find_route},
    util::{Coordinates, Date, DateTime, Location, Time},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut station_dir = StationDirectory::new();
    station_dir.load_from_gtfs("data/path/stops.txt", Some("path"))?;

    println!("\n**BUILD GRAPH**\n");
    let transit_graph = Graph::from_gtfs_dir("data/path", &station_dir)?;

    println!(
        "Built graph with {} origin stations and {} active calendar services!",
        transit_graph.adjacency_list.len(),
        transit_graph.calendar.services.len()
    );

    let origin = Location::Point(Coordinates::new(40.730009, -74.034637)); // Newport / Jersey City
    let destination = Location::Point(Coordinates::new(40.7176003, -73.9863546)); // -73.9863546 Gym, -73.9858367 315 Park Ave S, Manhattan

    let weekday_query = Query {
        origin: origin.clone(),
        destination: destination.clone(),
        departure_time: DateTime::new(
            Date::new(2026, 8, 17),
            Time::from_minutes(19 * 60 + 15),
        ),
    };

    if let Some(plan) = find_route(&transit_graph, &station_dir, weekday_query) {
        println!("{}", plan);
    } else {
        println!("No route found.");
    }

    Ok(())
}
