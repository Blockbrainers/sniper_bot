use crate::models::config_models::Config;
use std::fs;
use std::path::Path;
use std::error::Error;

// Implement the function to load and parse the configuration file
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    let config_str = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&config_str)?;
    Ok(config)
}