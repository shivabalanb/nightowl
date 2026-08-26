use std::error::Error;

use nightowl::{
    bike_network::BikeNetwork,
    router::{Query, find_route},
    transit_network::TransitNetwork,
    util::{Coordinates, Location},
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n**LOAD NETWORKS**\n");
    let transit = TransitNetwork::from_gtfs_dir("data/path", Some("path"))?;
    let bike = BikeNetwork::from_gbfs("data/citibike", Some("citibike"))?;

    println!(
        "Transit Network: {} stations, {} graph nodes, {} calendar services",
        transit.stations.stations.len(),
        transit.graph.adjacency_list.len(),
        transit.schedule.services.len()
    );

    println!(
        "Bike Network:    {} Citi Bike docks loaded!\n",
        bike.stations.len()
    );

    let origin = Location::Point(Coordinates::new(40.72204775835277, -74.0368774356056)); // Home 
    let destination = Location::Point(Coordinates::new(40.71721004390394, -73.98642630279129)); // Vital LES

    // Defaults to right now (current local time)
    let query_now = Query::new(origin.clone(), destination.clone());

    println!("Querying with departure_time = None (defaults to right now: {}):", query_now.get_departure_time());
    if let Some(plan) = find_route(&transit, &bike, query_now) {
        println!("{}", plan);
    } else {
        println!("No route found for current time.");
    }

    Ok(())
}
