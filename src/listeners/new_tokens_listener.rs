use std::{sync::Arc, error::Error};

use ethers::{providers::{Provider, StreamExt, Ws}, abi::Address, types::BlockNumber};
use tokio::sync::Barrier;
// use futures::StreamExt; 

use crate::{
    models::config_models::{NetworkMetadata, ExchangeConfig},
    bindings::{uniswap_v2_factory::{UniswapV2Factory, PairCreatedFilter}, uniswap_v3_factory::{UniswapV3Factory, PoolCreatedFilter}},
    analysis::processor::process_pair
};

pub async fn listen_to_new_tokens(network_metadata: &NetworkMetadata, exchange: &ExchangeConfig, barrier: Arc<Barrier>) {
    let network_name = &network_metadata.name;
    let exchange_name = &exchange.name;

    let provider = Provider::<Ws>::connect(&network_metadata.ws_url).await.unwrap();
    let client = Arc::new(provider);

    log::trace!("[{} - {}] Task setup complete, waiting at barrier.", network_name, exchange_name);
    barrier.wait().await;
    log::info!("[{} - {}] Passed barrier, starting main workload.", network_name, exchange_name);

    match exchange.base_implementation.as_str() {
        "UniswapV2" => {
            match listen_to_uniswap_v2_new_pairs(client, network_metadata, exchange).await {
                Ok(_) => (),
                Err(e) => log::error!("[{} - {}] Error in listener: {}", network_name, exchange_name, e),
            }
        },
        "UniswapV3" => {
            match listen_to_uniswap_v3_new_pools(client, network_metadata, exchange).await {
                Ok(_) => (),
                Err(e) => log::error!("[{} - {}] Error in listener: {}", network_name, exchange_name, e),
            }
        },
        _ => log::error!("Unknown Exchange: {}", exchange_name),
    }
}

async fn listen_to_uniswap_v2_new_pairs(client_clone: Arc<Provider<Ws>>, network_metadata: &NetworkMetadata, exchange: &ExchangeConfig) -> Result<(), Box<dyn Error>> {
    let exchange_name = &exchange.name;
    let contract_address: Address = exchange.factory_contract_address.parse().expect("Invalid contract address");
    let contract = UniswapV2Factory::new(contract_address, client_clone);
    let events = contract.event::<PairCreatedFilter>().from_block(BlockNumber::Latest);
    let mut stream = events.stream().await.unwrap();
    let network_name = network_metadata.name.clone();
    
    log::info!("[{} - {}] Listening for events PairCreatedFilter contract: {}", network_name, exchange_name, contract_address);

    while let Some(event) = stream.next().await {
        match event {
            Ok(pair_created_event) => {
                log::warn!(
                    "[{} - {}] PairCreatedFilter event received: {:?}",
                    network_name, exchange_name, pair_created_event,
                );
                process_pair(
                    network_metadata,
                    exchange,
                    &pair_created_event.token_0,
                    &pair_created_event.token_1,
                    &pair_created_event.pair
                ).await?;
            },
            Err(e) => {
                log::error!("[{} - {}] Error listening for PairCreatedFilter events: {:?}", network_name, exchange_name, e);
            },
        }
    }

    Ok(())
}

async fn listen_to_uniswap_v3_new_pools(client_clone: Arc<Provider<Ws>>, network_metadata: &NetworkMetadata, exchange: &ExchangeConfig) -> Result<(), Box<dyn Error>> {
    let exchange_name = &exchange.name;
    let contract_address: Address = exchange.factory_contract_address.parse().expect("Invalid contract address");
    let contract = UniswapV3Factory::new(contract_address, client_clone);
    
    let events = contract.event::<PoolCreatedFilter>().from_block(BlockNumber::Latest);
    let mut stream = events.stream().await.unwrap();
    let network_name = network_metadata.name.clone();
    
    log::info!("[{} - {}] Listening for events PoolCreatedFilter contract: {}", network_name, exchange_name, contract_address);
    
    while let Some(event) = stream.next().await {
        match event {
            Ok(pair_created_event) => {
                log::warn!(
                    "[{} - {}] PoolCreatedFilter event received: {:?}",
                    network_name, exchange_name, pair_created_event,
                );
                process_pair(
                    network_metadata,
                    exchange,
                    &pair_created_event.token_0,
                    &pair_created_event.token_1,
                    &pair_created_event.pool
                ).await?;
            },
            Err(e) => {
                log::error!("[{} - {}] Error listening for PoolCreatedFilter events: {:?}", network_name, exchange_name, e);
            },
        }
    }

    Ok(())
}
