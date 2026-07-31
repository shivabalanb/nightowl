const EARTH_RADIUS_MILES: f64 = 3959.0;

pub fn equirectangular_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
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
