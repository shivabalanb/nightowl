use std::error::Error;

use clap::Parser;
use nightowl::{
    bike_network::BikeNetwork,
    realtime::NjtClient,
    router::{Query, find_route},
    transit_network::TransitNetwork,
    util::{Coordinates, Location, ModeSet, TransitMode},
};

#[derive(Parser, Debug)]
#[command(name = "nightowl", about = "Multimodal Transit Router for NYC & NJ")]
struct Args {
    /// Disable Citi Bike
    #[arg(long)]
    no_bike: bool,

    /// Disable PATH trains
    #[arg(long)]
    no_path: bool,

    /// Disable MTA Subway
    #[arg(long)]
    no_mta: bool,

    /// Disable Hudson-Bergen Light Rail
    #[arg(long)]
    no_hblr: bool,

    /// Explicit list of allowed modes (e.g. --modes walk,path,mta)
    #[arg(long, value_delimiter = ',')]
    modes: Option<Vec<TransitMode>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let _ = dotenvy::dotenv();
    let args = Args::parse();

    let mut modes = if let Some(custom) = args.modes {
        ModeSet::from_modes(&custom)
    } else {
        ModeSet::all()
    };

    if args.no_bike {
        modes.disable(TransitMode::Bike);
    }
    if args.no_path {
        modes.disable(TransitMode::Path);
    }
    if args.no_mta {
        modes.disable(TransitMode::Mta);
    }
    if args.no_hblr {
        modes.disable(TransitMode::Hblr);
    }

    println!("Loading networks...");
    let mut transit = TransitNetwork::from_gtfs_dir("data/path", Some("path"))?;
    transit.load_gtfs("data/hblr", Some("hblr"))?;
    transit.load_gtfs("data/mta", Some("mta"))?;
    let mut bike = BikeNetwork::from_gbfs("data/citibike", Some("citibike"))?;

    println!(
        "Networks ready: {} transit stations ({} graph nodes, {} services), {} Citi Bike docks",
        transit.stations.stations.len(),
        transit.graph.adjacency_list.len(),
        transit.schedule.services.len(),
        bike.stations.len()
    );
    println!("Active modes:   {}", modes.active_modes_string());

    let origin = Location::Point(Coordinates::new(40.72204775835277, -74.0368774356056)); // Jersey City Home
    let destination = Location::Point(Coordinates::new(40.71721004390394, -73.98642630279129)); // Vital LES

    let query_now = Query::new(origin.clone(), destination.clone()).with_modes(modes);
    let departure_time = query_now.get_departure_time();

    println!("Real-time sync:");

    // 2. Fetch live Citi Bike dock availability
    if query_now.modes.allows(TransitMode::Bike) {
        match bike.refresh_status() {
            Ok(count) => println!("  [OK] Citi Bike: synced {} live docks", count),
            Err(e) => println!("  [WARN] Citi Bike live unavailable ({}), using static", e),
        }
    } else {
        println!("  [SKIP] Citi Bike: mode disabled by filter");
    }

    // 3. Fetch live PATH train countdowns
    if query_now.modes.allows(TransitMode::Path) {
        match transit.refresh_realtime(&departure_time) {
            Ok(count) => println!("  [OK] PATH: synced {} live departures", count),
            Err(e) => println!("  [WARN] PATH live unavailable ({}), using static", e),
        }
    } else {
        println!("  [SKIP] PATH: mode disabled by filter");
    }

    // 4. NJ Transit client check
    if query_now.modes.allows(TransitMode::Hblr) {
        if let Some(mut njt) = NjtClient::from_env() {
            match njt.get_token() {
                Ok(_) => println!("  [OK] NJ Transit: authenticated user '{}'", njt.username),
                Err(e) => println!("  [WARN] NJ Transit auth failed: {}", e),
            }
        } else {
            println!("  [INFO] NJ Transit: no credentials configured in .env");
        }
    }

    if let Some(plan) = find_route(&transit, &bike, query_now) {
        println!("{}", plan);
    } else {
        println!("No route found for current time with selected modes.");
    }

    Ok(())
}
