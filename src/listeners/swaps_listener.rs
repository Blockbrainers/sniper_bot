use ethers::{
    providers::{Provider, StreamExt, Ws},
    types::{Address, BlockNumber, U256, I256},
    contract::Contract,
    core::types::Filter,
};
use std::sync::Arc;
use std::error::Error;

use crate::{
    bindings::{uniswap_v2_pair::{UniswapV2Pair, SwapFilter}, uniswap_v3_pool::{UniswapV3Pool, SwapFilter as SwapFilterV3}},
    models::config_models::{NetworkMetadata, ExchangeConfig}
};

pub async fn listen_to_swaps(
    network_metadata: &NetworkMetadata,
    exchange: &ExchangeConfig,
    token_address: Address,
    pair_address: Address
) {
    let network_name = &network_metadata.name;
    let exchange_name = &exchange.name;

    log::trace!("[{} - {}] Task setup complete, waiting at barrier.", network_name, exchange_name);
    log::info!("[{} - {}] Passed barrier, starting main workload.", network_name, exchange_name);

    let provider = Provider::<Ws>::connect(&network_metadata.ws_url).await.unwrap();
    let client = Arc::new(provider);

    match exchange.base_implementation.as_str() {
        "UniswapV2" => {
            match listen_to_swaps_v2(client, network_metadata, exchange, token_address, pair_address).await {
                Ok(_) => (),
                Err(e) => log::error!("[{} - {}] Error in listener: {}", network_name, exchange_name, e),
            }
        },
        "UniswapV3" => {
            match listen_to_swaps_v3(client, network_metadata, exchange, token_address, pair_address).await {
                Ok(_) => (),
                Err(e) => log::error!("[{} - {}] Error in listener: {}", network_name, exchange_name, e),
            }
        },
        _ => log::error!("Unknown Exchange: {}", exchange_name),
    }
}

async fn listen_to_swaps_v2(
    client_clone: Arc<Provider<Ws>>,
    network_metadata: &NetworkMetadata,
    exchange: &ExchangeConfig,
    token_address: Address,
    pair_address: Address
) -> Result<(), Box<dyn Error>> {
    let exchange_name = &exchange.name;
    let pair_address_clone = pair_address.clone();
    let pair_contract = UniswapV2Pair::new(pair_address_clone, client_clone);
    let events = pair_contract.event::<SwapFilter>().from_block(BlockNumber::Latest);
    let mut stream = events.stream().await.unwrap();
    let network_name = network_metadata.name.clone();
    
    log::info!("[{} - {}] Listening for events PairCreatedFilter contract: {}", network_name, exchange_name, pair_address);

    while let Some(swap) = stream.next().await {
        match swap {
            Ok(swap_event) => {
                // Determine if the token of interest is token0 or token1
                let is_token0 = pair_contract.token_0().await? == token_address;
    
                let amount_in = if is_token0 { swap_event.amount_0_in } else { swap_event.amount_1_in };
                let amount_out = if is_token0 { swap_event.amount_0_out } else { swap_event.amount_1_out };
    
                // A 'sell' of the token of interest occurs when there's an amount 'in' and no amount 'out'
                if amount_in > U256::from(0) && amount_out == U256::from(0) {
                    // This is a sell for the token of interest
                    log::info!("[{} - {}] Sell detected: {:?}", network_name, exchange_name, swap_event);
                    // Handle the sell event here (logging, notifications, further processing...)
                }
            },
            Err(e) => {
                log::error!("[{} - {}] Error listening for Swap events: {:?}", network_name, exchange_name, e);
            },
        }
    }

    Ok(())
}


async fn listen_to_swaps_v3(
    client_clone: Arc<Provider<Ws>>,
    network_metadata: &NetworkMetadata,
    exchange: &ExchangeConfig,
    token_address: Address,
    pair_address: Address
) -> Result<(), Box<dyn Error>> {
    let exchange_name = &exchange.name;
    let pair_contract = UniswapV3Pool::new(pair_address.clone(), client_clone.clone());
    let events = pair_contract.event::<SwapFilterV3>().from_block(BlockNumber::Latest);
    let mut stream = events.stream().await.unwrap();
    let network_name = network_metadata.name.clone();
    let wrapped_native_token = network_metadata.wrapped_native_address;

    // Retrieve token0 and token1 addresses from the pair
    let token0 = pair_contract.token_0().await?;
    let token1 = pair_contract.token_1().await?;

    log::info!("[{} - {}] Listening for Swap events on contract: {}", network_name, exchange_name, pair_address);

    while let Some(swap) = stream.next().await {
        match swap {
            Ok(swap_event) => {
                // Determine if the target token is token0 or token1
                let is_target_token0 = token0 == token_address && token0 != wrapped_native_token;
                let is_target_token1 = token1 == token_address && token1 != wrapped_native_token;

                // Determine the amount for the target token
                let amount_target_token = if is_target_token0 {
                    swap_event.amount_0
                } else if is_target_token1 {
                    swap_event.amount_1
                } else {
                    continue; // Neither token in the pair is the target token
                };

                // Check if it's a sell event of the target token
                if amount_target_token > I256::from(0) {
                    log::info!("[{} - {}] Sell of target token detected: {:?}", network_name, exchange_name, swap_event);
                    // Handle the sell event here
                }
            },
            Err(e) => {
                log::error!("[{} - {}] Error listening for Swap events: {:?}", network_name, exchange_name, e);
            },
        }
    }

    Ok(())
}
