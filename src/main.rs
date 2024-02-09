mod bindings;
mod models;
mod config;
mod errors;
mod network;
mod analysis;
mod listeners;
mod utils;
mod trading;

use ethers::{
    providers::{Provider, Ws},
};

use tokio::spawn;
use tokio::sync::Barrier;
use std::{sync::Arc};
use std::error::Error; 
use futures::future::join_all;
use log::LevelFilter;
use env_logger::Builder;

use dotenv::dotenv;
use std::env;

use crate::{config::load_config, listeners::new_tokens_listener::listen_to_new_tokens};
use std::path::Path;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the dotenv
    dotenv().ok();
    
    Builder::new()
        .filter(Some("sniper_bot"), LevelFilter::Info)
        .filter(Some("tokio"), LevelFilter::Warn)
        .init();

    let environment = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let config_file = match environment.as_str() {
        "production" => "config_prod.json",
        "local" => "config_local.json",
        _ => "config_dev.json",  // Default to development
    };

    let config = load_config(Path::new(config_file))?;
    println!("Loaded configuration for the {} environment.", environment);

    let total_exchanges = config.networks.iter().map(|network| network.exchanges.len()).sum::<usize>();
    let barrier = Arc::new(Barrier::new(total_exchanges + 1)); // +1 for the main thread

    let mut tasks = Vec::new();

    for network in config.networks.into_iter() {        
        for exchange in network.exchanges.into_iter() {
            // let provider = Provider::<Ws>::connect(&network.metadata.ws_url).await?;
            // let client = Arc::new(provider);
            let barrier_clone = barrier.clone();
            let network_metadata = network.metadata.clone();

            // Spawn each listener as a separate asynchronous task
            let task: tokio::task::JoinHandle<()> = spawn(async move {
                listen_to_new_tokens(&network_metadata, &exchange, barrier_clone).await;
            });
            tasks.push(task);
        }
    }

    // Wait for all tasks to reach the barrier point
    barrier.wait().await;

    // Wait for all tasks to complete
    join_all(tasks).await;

    log::info!("All tasks have completed.");
    Ok(())
}
