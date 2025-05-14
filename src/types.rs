use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub listener: String,  // Frontend listener address
    pub backends_by_location: Arc<Mutex<HashMap<String, Vec<String>>>>, // Backends grouped by location
    pub current_indices: Arc<Mutex<HashMap<String, usize>>>, // Round-robin index per location
}
