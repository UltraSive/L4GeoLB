use tokio::net::TcpListener;
use std::sync::Arc;
use crate::config::load_env;
use crate::endpoints::get_public_tcp_endpoints;
use crate::backend::handle_client;
use crate::location::{get_locations, rank_locations_by_proximity};

mod config;
mod endpoints;
mod backend;
mod types;
mod location;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_env(); // Load environment variables

    // Load the locations in the distance proximity order
    let current_location_id = config::get_current_location()?;
    println!("Current location ID: {}", current_location_id);

    let locations = get_locations().await?;
    println!("List locations: {:?}", locations);

    let current_location = locations.iter().find(|loc| loc.id == current_location_id)
        .ok_or("Current location not found in database")?;
    println!("Current location: {:?}", current_location);

    let ranked_locations = rank_locations_by_proximity(&locations, current_location);
    println!("Ranked locations: {:?}", ranked_locations);

    let location_ordered: Vec<String> = ranked_locations
        .into_iter()
        .map(|loc| loc.id)
        .collect();
    println!("Location order: {:?}", location_ordered);

    let location_order = Arc::new(location_ordered);

    // Load the endpoints
    let endpoints = get_public_tcp_endpoints().await?;
    println!("Public TCP Endpoints: {:?}", endpoints);

    let mut tasks = Vec::new();

    for endpoint in endpoints {
        let listener_address = endpoint.listener.clone();
        let backends_by_location = Arc::clone(&endpoint.backends_by_location);
        let current_indices = Arc::clone(&endpoint.current_indices);
        let location_order = Arc::clone(&location_order); // Clone Arc

        let listener = TcpListener::bind(&listener_address).await?;
        println!("Load balancer running on {}", listener.local_addr()?);

        let task = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((client_socket, _)) => {
                        tokio::spawn({
                            let backends_by_location = Arc::clone(&backends_by_location);
                            let current_indices = Arc::clone(&current_indices);
                            let location_order = Arc::clone(&location_order); // Clone Arc
                            async move {
                                if let Err(e) = handle_client(
                                    client_socket,
                                    backends_by_location,
                                    current_indices,
                                    location_order.clone(), // Pass location_order
                                )
                                .await
                                {
                                    eprintln!("Error handling client: {}", e);
                                }
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection on {}: {}", listener_address, e);
                        break;
                    }
                }
            }
        });

        tasks.push(task);
    }

    for task in tasks {
        if let Err(e) = task.await {
            eprintln!("Task error: {}", e);
        }
    }

    Ok(())
}
