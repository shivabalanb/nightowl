use petgraph::algo::dijkstra;
use petgraph::graph::UnGraph;

#[derive(Debug, Clone)]
pub struct Landmark {
    pub name: &'static str,
    pub lat: f64,
    pub lon: f64,
}

const EARTH_RADIUS_MILES: f64 = 3959.0;

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

fn main() {
    let mut graph = UnGraph::<Landmark, f64>::new_undirected();

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
    graph.add_edge(newport, grove_st, dist_newport_grove);

    let dist_grove_exchange = equirectangular_distance(
        graph[grove_st].lat,
        graph[grove_st].lon,
        graph[exchange_pl].lat,
        graph[exchange_pl].lon,
    );
    graph.add_edge(grove_st, exchange_pl, dist_grove_exchange);

    println!(
        "Edge: {} <-> {} ({:.4} miles)",
        graph[newport].name, graph[grove_st].name, dist_newport_grove
    );
    println!(
        "Edge: {} <-> {} ({:.4} miles)",
        graph[grove_st].name, graph[exchange_pl].name, dist_grove_exchange
    );
}
