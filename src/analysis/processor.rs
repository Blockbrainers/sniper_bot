use std::error::Error;

use ethers::types::H160;

use crate::{models::{config_models::{NetworkMetadata, ExchangeConfig}, processed_trade::{ProcessedTrade, TradeSubStatus}}, analysis::security_checker::assess_token_security, trading::buy::buy_token, listeners::{swaps_listener::listen_to_swaps, mempool_swap_listener::listen_to_mempool_swaps}};

const YOUR_CONFIDENCE_THRESHOLD: f64 = 70.0;

pub async fn process_pair(network_metadata: &NetworkMetadata, exchange: &ExchangeConfig, token_0: &H160, token_1: &H160, pair_or_pool: &H160) -> Result<(), Box<dyn Error>> {
    // Check if wrapped native address is neither token_0 nor token_1
    let exchange_name = exchange.name.clone();
    if *token_0 != network_metadata.wrapped_native_address && *token_1 != network_metadata.wrapped_native_address {
        log::info!(
            "[{} - {}] Pair or pool does not involve the wrapped native token. This might indicate it's not a new liquidity pool with the native token as a base.",
            network_metadata.name, exchange_name
        );
        return Ok(());
    }
    
    // Determine which token to assess
    let token_to_assess = if *token_0 == network_metadata.wrapped_native_address { token_1 } else { token_0 };
    
    log::info!("[{} - {} - {}] Processing pair... Pair/Pool address: {}", network_metadata.name, exchange_name, token_to_assess, pair_or_pool);
    let mut trade = ProcessedTrade::new(
        network_metadata.chain_id.clone(),
        network_metadata.name.clone(),
        exchange.name.clone(),
        *pair_or_pool, // Use the pair_or_pool address as the pair_address
        *token_0,     // Use token_0 address as the token_address
        network_metadata.wrapped_native_address, // Use the wrapped native address as the base_token_address
    )?;
    
    // Safety checks
    let assessment = assess_token_security(network_metadata, &exchange_name, token_to_assess).await?;
    log::info!("[{} - {} - {}]  Confidence Score: {}", network_metadata.name, exchange_name, token_to_assess, assessment.confidence_score,);
    log::info!("[{} - {} - {}]  Recommended Trade Amount: {}", network_metadata.name, exchange_name, token_to_assess, assessment.recommended_trade_amount);

    let network_metadata_clone = network_metadata.clone();
    let exchange_clone = exchange.clone();
    let token_to_assess_clone = *token_to_assess;

    if assessment.confidence_score >= YOUR_CONFIDENCE_THRESHOLD {
        // Call the buy function with cloned data
        let buy_result = buy_token(
            &exchange_clone,
            &network_metadata_clone,
            token_to_assess_clone,
            assessment.recommended_trade_amount,
            exchange_clone.base_implementation == "UniswapV3",
            None
        ).await;

        match buy_result {
            Ok(tx_hash) => {
                log::info!("Successfully bought the token. Transaction hash: {:?}", tx_hash);
                // trade.open_position(TradeSubStatus::FailedSecurityCheck);
                // Spawn a new task for listen_to_swaps with cloned data
                // Clone data for the first listener
                let network_metadata_clone1 = network_metadata.clone();
                let exchange_clone1 = exchange.clone();
                let token_to_assess_clone1 = *token_to_assess;
                let pair_or_pool_clone1 = *pair_or_pool;

                // Spawn a new task for listen_to_swaps
                tokio::spawn(async move {
                    listen_to_swaps(&network_metadata_clone1, &exchange_clone1, token_to_assess_clone1, pair_or_pool_clone1).await;
                });

                // Clone data for the second listener
                let network_metadata_clone2 = network_metadata.clone();
                let exchange_clone2 = exchange.clone();
                let token_to_assess_clone2 = *token_to_assess;
                let pair_or_pool_clone2 = *pair_or_pool;

                // Spawn a new task for listen_to_mempool_swaps
                tokio::spawn(async move {
                    listen_to_mempool_swaps(&network_metadata_clone2, &exchange_clone2, token_to_assess_clone2, pair_or_pool_clone2).await;
                });
            },
            Err(e) => log::error!("Failed to buy the token: {}", e),
        }
    } else {
        trade.canceled(TradeSubStatus::FailedSecurityCheck);
        log::warn!("Confidence score is too low. Skipping trade.");
    }

    Ok(())
}