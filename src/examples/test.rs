use petgraph::algo::dijkstra;
use petgraph::graph::DiGraph;

use crate::{
    math::equirectangular_distance,
    model::{EdgeType, Landmark, TrainDeparture},
};

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
                travel_time_miles: 0.64,
            },
            TrainDeparture {
                time_of_day: 510,
                travel_time_miles: 0.64,
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
