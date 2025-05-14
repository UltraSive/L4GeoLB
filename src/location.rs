use crate::config;
use tokio_postgres::NoTls;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Location {
    pub id: String,
    pub latitude: f64,
    pub longitude: f64,
}

pub fn to_radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

pub fn haversine(coord1: (f64, f64), coord2: (f64, f64)) -> f64 {
    let (lat1, lon1) = coord1;
    let (lat2, lon2) = coord2;

    let r = 6371.0; // Earth's radius in kilometers

    let delta_lat = to_radians(lat2 - lat1);
    let delta_lon = to_radians(lon2 - lon1);

    let lat1 = to_radians(lat1);
    let lat2 = to_radians(lat2);

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    r * c
}

pub async fn get_locations() -> Result<Vec<Location>, Box<dyn Error>> {
    let conn_str = config::get_database_url()?;
    println!("Connecting to database: {}", conn_str);

    let (client, connection) = tokio_postgres::connect(&conn_str, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let rows = client
        .query(
            "SELECT id, latitude, longitude FROM \"public\".\"Location\";",
            &[],
        )
        .await?;

    let locations = rows
        .iter()
        .map(|row| Location {
            id: row.get("id"),
            latitude: row.get("latitude"),
            longitude: row.get("longitude"),
        })
        .collect();

    Ok(locations)
}

pub fn rank_locations_by_proximity(
    locations: &[Location],
    reference: &Location,
) -> Vec<Location> {
    let mut ranked: Vec<(Location, f64)> = locations
        .iter()
        .map(|loc| {
            let distance = haversine(
                (reference.latitude, reference.longitude),
                (loc.latitude, loc.longitude),
            );
            (loc.clone(), distance)
        })
        .collect();

    ranked.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    ranked.into_iter().map(|(loc, _)| loc).collect()
}