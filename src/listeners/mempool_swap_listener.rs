use ethers::{
    providers::{Provider, StreamExt, Middleware, Ws},
    types::{TransactionRequest, U256, Address},
};
use std::sync::Arc;
use std::error::Error;

use crate::models::config_models::{NetworkMetadata, ExchangeConfig};

// This function listens to the mempool for pending swap transactions related to a specific pair or token
pub async fn listen_to_mempool_swaps(
    network_metadata: &NetworkMetadata,
    exchange: &ExchangeConfig,
    token_address: Address,
    pair_address: Address
) -> Result<(), Box<dyn Error>> {
    let network_name = &network_metadata.name;
    let exchange_name = &exchange.name;

    // Connect to an Ethereum provider that allows you to query the mempool
    let provider = Provider::<Ws>::connect(&network_metadata.ws_url).await.unwrap();

    log::info!("[{} - {}] Listening to mempool for swaps involving Pair/Token: {}", network_name, exchange_name, pair_address);

    // Subscribe to the mempool (new pending transactions)
    let mut stream = provider.watch_pending_transactions().await?;
    
    while let Some(tx_hash) = stream.next().await {
        match provider.get_transaction(tx_hash).await {
            Ok(Some(tx_details)) => {
                // Transaction details successfully fetched
                log::info!("[{} - {}] Transaction details: {:?}", network_name, exchange_name, tx_details);
                if let Some(to_address) = tx_details.to {
                    if to_address == pair_address {
                        // Transaction is to the pair_address; process it
                        log::info!("[{} - {}] Relevant transaction to pair_address found: {:?}", network_name, exchange_name, tx_details);
                        // ... your analysis and reaction logic here ...
                    }
                }
                // ... your analysis and reaction logic here ...
            },
            Ok(None) => {
                log::warn!("[{} - {}] Transaction details not found for hash: {:?}", network_name, exchange_name, tx_hash);
            },
            Err(e) => {
                log::error!("[{} - {}] Error fetching transaction details: {:?}", network_name, exchange_name, e);
            }
        }
    }

    Ok(())
}
