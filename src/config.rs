use dotenv::dotenv;
use std::env;

pub fn load_env() {
    dotenv().ok(); // Load environment variables
}

pub fn get_database_url() -> Result<String, Box<dyn std::error::Error>> {
    env::var("DATABASE_URL").map_err(|e| e.into())
}

pub fn get_current_location() -> Result<String, Box<dyn std::error::Error>> {
  env::var("LOCATION").map_err(|e| e.into())
}