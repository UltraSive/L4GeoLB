use crate::config;
use crate::types::Endpoint;
use tokio_postgres::NoTls;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

pub async fn get_public_tcp_endpoints() -> Result<Vec<Endpoint>, Box<dyn std::error::Error>> {
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
            "SELECT e.port AS endpoint_port, e.external_address_id AS endpoint_external_address_id, e.external_port AS endpoint_external_port,
            a.id AS app_id, a.project_id AS project_id, p.location_id AS placement_location_id
            FROM \"public\".\"Endpoint\" e
            JOIN \"public\".\"App\" a ON e.app_id = a.id
            LEFT JOIN \"public\".\"Placement\" p ON a.id = p.app_id
            WHERE e.external_address_id IS NOT NULL
            AND e.external_port IS NOT NULL;",
            &[],
        )
        .await?;

    let mut listeners_map: HashMap<String, Arc<Mutex<HashMap<String, Vec<String>>>>> = HashMap::new();
    let mut endpoints = Vec::new();

    for row in rows {
        let listener_address = format!(
            "{}:{}",
            row.get::<&str, String>("endpoint_external_address_id"),
            row.get::<&str, i32>("endpoint_external_port")
        );

        let existing_backends_by_location = listeners_map
            .entry(listener_address.clone())
            .or_insert_with(|| Arc::new(Mutex::new(HashMap::new())));

        if let Some(location_id) = row.get::<&str, Option<String>>("placement_location_id") {
            let backend_address = format!(
                "app-{}-{}-svc.proj-{}.svc.cluster.local:{}",
                row.get::<&str, String>("app_id"),
                location_id,
                row.get::<&str, String>("project_id"),
                row.get::<&str, i32>("endpoint_port")
            );

            let mut locked_backends = existing_backends_by_location.lock().await;
            locked_backends
                .entry(location_id)
                .or_insert_with(Vec::new)
                .push(backend_address);
        }

        if !endpoints.iter().any(|e: &Endpoint| e.listener == listener_address) {
            endpoints.push(Endpoint {
                listener: listener_address.clone(),
                backends_by_location: Arc::clone(&existing_backends_by_location),
                current_indices: Arc::new(Mutex::new(HashMap::new())),
            });
        }
    }

    Ok(endpoints)
}
