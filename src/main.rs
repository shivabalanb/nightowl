use std::error::Error;

use nightowl::{
    router::{Query, find_route},
    transit_network::TransitNetwork,
    util::{Coordinates, Date, DateTime, Location, Time},
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n**LOAD TRANSIT NETWORK**\n");
    let transit = TransitNetwork::from_gtfs_dir("data/path", Some("path"))?;

    println!(
        "Built transit network with {} stations, {} graph nodes, and {} calendar services!",
        transit
            .stations
            .find_nearby_stations(&Coordinates::new(40.73, -74.03), 100.0)
            .len(),
        transit.graph.adjacency_list.len(),
        transit.schedule.services.len()
    );

    let origin = Location::Point(Coordinates::new(40.730009, -74.034637)); // Newport / Jersey City
    let destination = Location::Point(Coordinates::new(40.7176003, -73.9863546)); // -73.9863546 Gym, -73.9858367 315 Park Ave S, Manhattan

    let weekday_query = Query {
        origin: origin.clone(),
        destination: destination.clone(),
        departure_time: DateTime::new(Date::new(2026, 8, 17), Time::from_minutes(19 * 60 + 15)),
    };

    if let Some(plan) = find_route(&transit, weekday_query) {
        println!("{}", plan);
    } else {
        println!("No route found.");
    }

    Ok(())
}
