use tokio::io::{self, copy_bidirectional};
use tokio::net::TcpStream;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

pub async fn handle_client(
  mut client_socket: TcpStream,
  backends_by_location: Arc<Mutex<HashMap<String, Vec<String>>>>,
  current_indices: Arc<Mutex<HashMap<String, usize>>>,
  location_order: Arc<Vec<String>>, // Accept location_order as Arc
) -> io::Result<()> {
  match select_backend(&backends_by_location, &current_indices, &location_order).await {
      Ok(backend_address) => {
          match TcpStream::connect(&backend_address).await {
              Ok(mut backend_socket) => {
                  println!("Successfully connected to backend: {}", backend_address);
                  forward_traffic(&mut client_socket, &mut backend_socket).await?;
              }
              Err(e) => {
                  eprintln!("Failed to connect to backend: {}", e);
                  return Err(e);
              }
          }
      }
      Err(e) => {
          eprintln!("No suitable backend found: {}", e);
          return Err(e);
      }
  }

  Ok(())
}

async fn select_backend(
  backends_by_location: &Arc<Mutex<HashMap<String, Vec<String>>>>,
  current_indices: &Arc<Mutex<HashMap<String, usize>>>,
  location_order: &Arc<Vec<String>>, // Accept location_order
) -> io::Result<String> {
  let backends_by_location = backends_by_location.lock().await;
  let mut current_indices = current_indices.lock().await;

  for location in location_order.iter() {
      if let Some(backends) = backends_by_location.get(location) {
          if !backends.is_empty() {
              let index = current_indices.entry(location.clone()).or_insert(0);
              let backend = &backends[*index];
              *index = (*index + 1) % backends.len();
              return Ok(backend.clone());
          }
      }
  }

  Err(io::Error::new(
      io::ErrorKind::NotFound,
      "No suitable backends available",
  ))
}

async fn forward_traffic(
    client_socket: &mut TcpStream,
    backend_socket: &mut TcpStream,
) -> io::Result<()> {
    let result = copy_bidirectional(client_socket, backend_socket).await;

    if result.is_ok() {
        println!("Traffic forwarding completed successfully.");
    } else {
        eprintln!("Error occurred during traffic forwarding: {:?}", result);
    }

    result?;
    Ok(())
}
