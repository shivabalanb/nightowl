use std::{
    error::Error,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Duration,
};

use axum::{
    Json, Router,
    extract::{Query as AxumQuery, State},
    http::Method,
    response::Html,
    routing::get,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

use nightowl::{
    bike_network::BikeNetwork,
    realtime::NjtClient,
    router::{Plan, Query, RouteStats, find_route},
    transit_network::TransitNetwork,
    util::{Coordinates, DateTime, Location, ModeSet, Time, TransitMode},
};

pub struct AppState {
    pub transit: RwLock<TransitNetwork>,
    pub bike: RwLock<BikeNetwork>,
}

#[derive(Debug, Deserialize)]
pub struct RouteParams {
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub origin_lat: Option<f64>,
    pub origin_lon: Option<f64>,
    pub dest_lat: Option<f64>,
    pub dest_lon: Option<f64>,
    pub departure_time: Option<String>,
    pub modes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RouteResponse {
    pub success: bool,
    pub plan: Option<Plan>,
    pub stats: Option<RouteStats>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub transit_stations_count: usize,
    pub bike_docks_count: usize,
}

#[derive(Debug, Serialize)]
pub struct StationMarker {
    pub id: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub kind: String,
    pub available_bikes: Option<u32>,
    pub available_docks: Option<u32>,
}

fn load_initial_state() -> Result<AppState, Box<dyn Error>> {
    let _ = dotenvy::dotenv();

    println!("Loading networks...");
    let mut transit = TransitNetwork::from_gtfs_dir("data/path", Some("path"))?;
    transit.load_gtfs("data/hblr", Some("hblr"))?;
    transit.load_gtfs("data/mta", Some("mta"))?;
    let mut bike = BikeNetwork::from_gbfs("data/citibike", Some("citibike"))?;

    println!(
        "Networks ready: {} transit stations, {} Citi Bike docks",
        transit.stations.stations.len(),
        bike.stations.len()
    );

    // Initial real-time sync
    let now = DateTime::now();
    let _ = bike.refresh_status();
    let _ = transit.refresh_realtime(&now);
    if let Some(mut njt) = NjtClient::from_env() {
        let _ = njt.get_token();
    }

    Ok(AppState {
        transit: RwLock::new(transit),
        bike: RwLock::new(bike),
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let app_state = load_initial_state()?;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(run_server(app_state))
}

async fn run_server(app_state: AppState) -> Result<(), Box<dyn Error>> {
    let state = Arc::new(app_state);

    // Background task to refresh real-time feeds every 30 seconds using spawn_blocking
    let bg_state = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            let s = bg_state.clone();
            let _ = tokio::task::spawn_blocking(move || {
                let current = DateTime::now();
                if let Ok(mut bike_lock) = s.bike.write() {
                    let _ = bike_lock.refresh_status();
                }
                if let Ok(mut transit_lock) = s.transit.write() {
                    let _ = transit_lock.refresh_realtime(&current);
                }
            }).await;
        }
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(serve_ui))
        .route("/api/health", get(handle_health))
        .route("/api/route", get(handle_route))
        .route("/api/stations", get(handle_stations))
        .layer(cors)
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    println!("Nightowl API & Web Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let transit = state.transit.read().unwrap();
    let bike = state.bike.read().unwrap();

    Json(HealthResponse {
        status: "ok".to_string(),
        transit_stations_count: transit.stations.stations.len(),
        bike_docks_count: bike.stations.len(),
    })
}

async fn handle_stations(State(state): State<Arc<AppState>>) -> Json<Vec<StationMarker>> {
    let transit = state.transit.read().unwrap();
    let bike = state.bike.read().unwrap();
    let mut list = Vec::new();

    for s in transit.stations.stations.values() {
        let kind = if s.id.starts_with("path:") {
            "path"
        } else if s.id.starts_with("mta:") {
            "mta"
        } else if s.id.starts_with("hblr:") {
            "hblr"
        } else {
            "transit"
        };

        list.push(StationMarker {
            id: s.id.clone(),
            name: s.name.clone(),
            lat: s.coordinates.lat,
            lon: s.coordinates.lon,
            kind: kind.to_string(),
            available_bikes: None,
            available_docks: None,
        });
    }

    for b in bike.stations.values() {
        let status = bike.status.get(&b.id);
        list.push(StationMarker {
            id: b.id.clone(),
            name: b.name.clone(),
            lat: b.coordinates.lat,
            lon: b.coordinates.lon,
            kind: "citibike".to_string(),
            available_bikes: status.map(|s| s.num_bikes_available),
            available_docks: status.map(|s| s.num_docks_available),
        });
    }

    Json(list)
}

async fn handle_route(
    State(state): State<Arc<AppState>>,
    AxumQuery(params): AxumQuery<RouteParams>,
) -> Json<RouteResponse> {
    // 1. Resolve Origin Coordinates
    let origin_coords = if let (Some(lat), Some(lon)) = (params.origin_lat, params.origin_lon) {
        Coordinates::new(lat, lon)
    } else if let Some(o_str) = params.origin {
        let res = tokio::task::spawn_blocking(move || {
            Coordinates::parse_or_geocode(&o_str).map_err(|e| e.to_string())
        })
        .await;
        match res {
            Ok(Ok(coords)) => coords,
            Ok(Err(e)) => {
                return Json(RouteResponse {
                    success: false,
                    plan: None,
                    stats: None,
                    error: Some(format!("Could not resolve origin: {}", e)),
                });
            }
            Err(e) => {
                return Json(RouteResponse {
                    success: false,
                    plan: None,
                    stats: None,
                    error: Some(format!("Geocoding task failed: {}", e)),
                });
            }
        }
    } else {
        return Json(RouteResponse {
            success: false,
            plan: None,
            stats: None,
            error: Some("Missing origin parameter (address or lat/lon).".to_string()),
        });
    };

    // 2. Resolve Destination Coordinates
    let dest_coords = if let (Some(lat), Some(lon)) = (params.dest_lat, params.dest_lon) {
        Coordinates::new(lat, lon)
    } else if let Some(d_str) = params.destination {
        let res = tokio::task::spawn_blocking(move || {
            Coordinates::parse_or_geocode(&d_str).map_err(|e| e.to_string())
        })
        .await;
        match res {
            Ok(Ok(coords)) => coords,
            Ok(Err(e)) => {
                return Json(RouteResponse {
                    success: false,
                    plan: None,
                    stats: None,
                    error: Some(format!("Could not resolve destination: {}", e)),
                });
            }
            Err(e) => {
                return Json(RouteResponse {
                    success: false,
                    plan: None,
                    stats: None,
                    error: Some(format!("Geocoding task failed: {}", e)),
                });
            }
        }
    } else {
        return Json(RouteResponse {
            success: false,
            plan: None,
            stats: None,
            error: Some("Missing destination parameter (address or lat/lon).".to_string()),
        });
    };

    let origin = Location::Point(origin_coords);
    let destination = Location::Point(dest_coords);

    let mut modes = if let Some(m_str) = params.modes {
        let mut set = ModeSet::from_modes(&[]);
        for part in m_str.split(',') {
            match part.trim().to_lowercase().as_str() {
                "walk" => set.enable(TransitMode::Walk),
                "bike" => set.enable(TransitMode::Bike),
                "path" => set.enable(TransitMode::Path),
                "mta" => set.enable(TransitMode::Mta),
                "hblr" => set.enable(TransitMode::Hblr),
                _ => {}
            }
        }
        set
    } else {
        ModeSet::all()
    };

    modes.enable(TransitMode::Walk);

    let mut query = Query::new(origin, destination).with_modes(modes);

    if let Some(dep_str) = params.departure_time {
        if let Ok(time) = dep_str.parse::<Time>() {
            let mut dt = DateTime::now();
            dt.time = time;
            query = query.with_departure_time(dt);
        }
    }

    let transit = state.transit.read().unwrap();
    let bike = state.bike.read().unwrap();

    if let Some(plan) = find_route(&transit, &bike, query) {
        let stats = plan.stats();
        Json(RouteResponse {
            success: true,
            plan: Some(plan),
            stats: Some(stats),
            error: None,
        })
    } else {
        Json(RouteResponse {
            success: false,
            plan: None,
            stats: None,
            error: Some("No route found with selected parameters.".to_string()),
        })
    }
}

async fn serve_ui() -> Html<&'static str> {
    Html(include_str!("ui/index.html"))
}
