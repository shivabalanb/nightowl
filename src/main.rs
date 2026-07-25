use std::thread::current;

use petgraph::algo::dijkstra;
use petgraph::graph::DiGraph;

#[derive(Debug, Clone)]
pub struct Landmark {
    pub name: &'static str,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone)]
pub struct TrainDeparture {
    pub time_of_day: u32, // minutes since midnight
    pub travel_time_miles: f64,
}

#[derive(Debug, Clone)]
pub enum EdgeType {
    Walk(f64),
    Transit(Vec<TrainDeparture>),
}

impl EdgeType {
    pub fn min_weight(&self) -> f64 {
        match self {
            EdgeType::Walk(dist) => *dist,
            EdgeType::Transit(departures) => departures
                .iter()
                .map(|d| d.travel_time_miles)
                .fold(f64::INFINITY, f64::min),
        }
    }
    pub fn weight_at_time(&self, current_time: u32) -> f64 {
        match self {
            EdgeType::Walk(dist_miles) => {
                // 3 mph walking speed = 1 mile / 20 mins
                dist_miles * 20.0
            }
            EdgeType::Transit(departures) => {
                let next_train = departures
                    .iter()
                    .filter(|d| d.time_of_day >= current_time)
                    .min_by_key(|d| d.time_of_day);
                match next_train {
                    Some(train) => {
                        let wait_time = (train.time_of_day - current_time) as f64;
                        // assume PATH trains travel ~25mph
                        let ride_time = train.travel_time_miles / (25.0 / 60.0);

                        wait_time + ride_time
                    }
                    None => f64::INFINITY,
                }
            }
        }
    }
}

const EARTH_RADIUS_MILES: f64 = 3959.0;

fn main() {
    let mut graph = DiGraph::<Landmark, EdgeType>::new();

    let newport = graph.add_node(Landmark {
        name: "Newport PATH",
        lat: 40.7268,
        lon: -74.0345,
    });

    let grove_st = graph.add_node(Landmark {
        name: "Grove St PATH",
        lat: 40.7196,
        lon: -74.0431,
    });

    let exchange_pl = graph.add_node(Landmark {
        name: "Exchange Place PATH",
        lat: 40.7162,
        lon: -74.0330,
    });

    let dist_newport_grove = equirectangular_distance(
        graph[newport].lat,
        graph[newport].lon,
        graph[grove_st].lat,
        graph[grove_st].lon,
    );

    graph.add_edge(newport, grove_st, EdgeType::Walk(dist_newport_grove));
    graph.add_edge(grove_st, newport, EdgeType::Walk(dist_newport_grove));

    graph.add_edge(
        newport,
        grove_st,
        EdgeType::Transit(vec![
            TrainDeparture {
                time_of_day: 480,
                travel_time_miles: 0.7366,
            },
            TrainDeparture {
                time_of_day: 510,
                travel_time_miles: 0.7366,
            },
        ]),
    );

    let dist_grove_exchange = equirectangular_distance(
        graph[grove_st].lat,
        graph[grove_st].lon,
        graph[exchange_pl].lat,
        graph[exchange_pl].lon,
    );

    graph.add_edge(grove_st, exchange_pl, EdgeType::Walk(dist_grove_exchange));
    graph.add_edge(exchange_pl, grove_st, EdgeType::Walk(dist_grove_exchange));

    graph.add_edge(
        grove_st,
        exchange_pl,
        EdgeType::Transit(vec![
            TrainDeparture {
                time_of_day: 480,
                travel_time_miles: 0.5366,
            },
            TrainDeparture {
                time_of_day: 510,
                travel_time_miles: 0.5366,
            },
        ]),
    );

    let dist_newport_exchange = equirectangular_distance(
        graph[newport].lat,
        graph[newport].lon,
        graph[exchange_pl].lat,
        graph[exchange_pl].lon,
    );

    graph.add_edge(newport, exchange_pl, EdgeType::Walk(dist_newport_exchange));
    graph.add_edge(exchange_pl, newport, EdgeType::Walk(dist_newport_exchange));

    graph.add_edge(
        newport,
        exchange_pl,
        EdgeType::Transit(vec![
            TrainDeparture {
                time_of_day: 480,
                travel_time_miles: 0.6366,
            },
            TrainDeparture {
                time_of_day: 510,
                travel_time_miles: 0.6366,
            },
        ]),
    );

    let path_costs = dijkstra(&graph, newport, Some(exchange_pl), |edge| {
        edge.weight().min_weight()
    });

    // Retrieve a reference to the Newport -> Exchange Place transit edge
    let newport_exchange_edge = &graph[graph.find_edge(newport, exchange_pl).unwrap()];

    println!("\n--- Testing Edge Weight At Specific Departure Times ---");
    println!(
        "Arriving at 8:00 AM (480 mins): {:.2} minutes total cost",
        newport_exchange_edge.weight_at_time(480)
    );
    println!(
        "Arriving at 8:10 AM (490 mins): {:.2} minutes total cost",
        newport_exchange_edge.weight_at_time(490)
    );
    println!(
        "Arriving at 9:00 AM (540 mins): {:.2} minutes total cost",
        newport_exchange_edge.weight_at_time(540)
    );
    println!("-------------------------------------------------------\n");

    if let Some(&shortest_distance) = path_costs.get(&exchange_pl) {
        println!(
            "\nSUCCESS: Shortest path distance from {} to {} is {:.4} miles!",
            graph[newport].name, graph[exchange_pl].name, shortest_distance
        );
    } else {
        println!("\nERROR: No path found between those stations.");
    };
}

fn equirectangular_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let lat1_rad = lat1.to_radians();
    let lon1_rad = lon1.to_radians();
    let lat2_rad = lat2.to_radians();
    let lon2_rad = lon2.to_radians();

    let delta_lat = lat1_rad - lat2_rad;
    let mean_lat = (lat1_rad + lat2_rad) / 2.0;
    let delta_lon = lon2_rad - lon1_rad;

    let x = delta_lon * mean_lat.cos() * EARTH_RADIUS_MILES;
    let y = delta_lat * EARTH_RADIUS_MILES;

    (x * x + y * y).sqrt()
}
