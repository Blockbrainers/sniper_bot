use std::{error::Error};

use crate::{
    models::config_models::{NetworkMetadata, ExchangeConfig},
    bindings::{uniswap_v2_router02::UniswapV2Router02, uniswap_v3_smart_router::{UniswapV3SmartRouter, ExactInputSingleParams}},
    network::{client::{create_client_arc}, transaction::{send_tx, send_tx_flashbots}}
};

use ethers::{
    prelude::*,
    types::{Address, U256},
    utils::parse_ether, providers::Middleware, core::k256::{ecdsa::SigningKey, Secp256k1},
};
use chrono::Utc;

const SLIPPAGE: f64 = 0.20;

pub async fn buy_token(
    exchange: &ExchangeConfig,
    network_metadata: &NetworkMetadata,
    target_token_address: Address,
    amount_in_eth: f64,
    is_v3: bool,
    fee: Option<u32>,
) -> Result<H256, Box<dyn Error>> {
    if is_v3 {
        buy_v3(
            exchange,
            network_metadata,
            target_token_address,
            amount_in_eth,
            fee,
        )
        .await
    } else {
        buy_v2(
            exchange,
            network_metadata,
            target_token_address,
            amount_in_eth,
        )
        .await
    }
}

async fn buy_v2(
    exchange: &ExchangeConfig,
    network_metadata: &NetworkMetadata,
    target_token_address: Address,
    amount_in_eth: f64,
) -> Result<H256, Box<dyn Error>> {
    // Initialize the Uniswap V2 Router
    let client_arc = create_client_arc(network_metadata.rpc_url.as_str(), network_metadata.clone().chain_id).await?;
    let router_contract_address: Address = exchange.router_contract_address.parse()?;
    let uniswap_v2_router = UniswapV2Router02::new(router_contract_address, client_arc.clone());
    let base_token_address: Address = network_metadata.wrapped_native_address;
    
     // Set up the transaction request for the swap
     let deadline = Utc::now().timestamp() as u64 + 15 * 60; // 15 minutes * 60 seconds
     let deadline_u256: U256 = U256::from(deadline);
     // Construct the path for the swap
     let path: Vec<Address> = vec![network_metadata.wrapped_native_address, target_token_address];
     // Convert the ETH amount to Wei
     let amount_in_wei: U256 = parse_ether(amount_in_eth.to_string().as_str())?.into();
     println!("-----> amount_in_wei: {}", amount_in_wei);
     // raw amount
     let amount_out_min: U256 = calculate_amount_out_v2(
        &uniswap_v2_router,
        amount_in_wei,
        base_token_address,
        target_token_address,
    )
    .await?;
    println!("-----> amount_out_min: {}", amount_out_min);
     // Prepare the function call for Uniswap V2 swap operation
     let function_call = uniswap_v2_router.swap_exact_eth_for_tokens(
         amount_out_min,
         path,
         network_metadata.wallet_address, // recipient address
         deadline_u256,
     );
     println!("************** function_call passed");
     // Estimate the gas for the transaction
    //  let estimated_gas = function_call.estimate_gas().await?;
    //  println!("************** estimated_gas: {}:", estimated_gas);
    //  // You may also want to fetch the current gas price from the network or use a strategy for setting it
    //  let gas_price = client_arc.get_gas_price().await?;
    //  println!("************** gas_price: {}:", gas_price);
    // // Convert chain_id from String to U64
    let chain_id: U64 = network_metadata.chain_id.parse()?;
 
     // Create the TransactionRequest manually
     let tx_request = TransactionRequest {
        chain_id: Some(chain_id),
        from: Some(network_metadata.wallet_address.into()),
        to: Some(NameOrAddress::Address(router_contract_address)),
        gas: None,
        gas_price:None,
        value: Some(amount_in_wei), // This is the ETH amount you're sending
        data: Some(function_call.tx.data().unwrap().clone()), // Extract the data from the function call
        // Nonce is typically managed by the client, but you can specify it manually if needed
        nonce: None, // Set this to Some(nonce) if you're manually managing nonces
     };

    // Check the chain ID to decide between Flashbots and regular sending
    if network_metadata.chain_id == "1" {
        // Send the transaction with Flashbots
        send_tx_flashbots(
            network_metadata,
            tx_request,
        ).await
    } else {
        // Send the transaction with the regular Ethereum client
        send_tx(
            client_arc,
            tx_request,
        ).await
    }
}

async fn buy_v3(
    exchange: &ExchangeConfig,
    network_metadata: &NetworkMetadata,
    target_token_address: Address,
    amount_in_eth: f64,
    fee: Option<u32>,
) -> Result<H256, Box<dyn Error>> {

    let client_arc = create_client_arc(network_metadata.rpc_url.as_str(), network_metadata.chain_id.clone()).await?;
    let router_contract_address: Address = exchange.router_contract_address.parse()?;
    let base_token_address: Address = network_metadata.wrapped_native_address;
    let recipient_address: Address = network_metadata.wallet_address;
    let uniswap_v3_router = UniswapV3SmartRouter::new(router_contract_address, client_arc.clone());

    let default_fee = 3000; // Default to a common fee tier, e.g., 0.3%
    let fee = fee.unwrap_or(default_fee);

    // Convert the ETH amount to Wei
    let amount_in_wei: U256 = parse_ether(amount_in_eth.to_string().as_str())?.into();
    // raw amount
    let amount_out_min = calculate_amount_out_v3(
        &uniswap_v3_router,
        recipient_address,
        amount_in_wei,
        base_token_address,
        target_token_address,
        fee
    )
    .await?;

    // Generate the Uniswap V3 transaction using bindings
    let params = ExactInputSingleParams {
        token_in: base_token_address,
        token_out: target_token_address,
        fee,
        recipient: recipient_address,
        amount_in: amount_in_wei,
        amount_out_minimum: amount_out_min, // Include the amount_out_minimum field
        sqrt_price_limit_x96: U256::zero(), // Set to zero or another appropriate value
    };

    let function_call = uniswap_v3_router.exact_input_single(params);
     // Estimate the gas for the transaction
     let estimated_gas = function_call.estimate_gas().await?;
 
     // You may also want to fetch the current gas price from the network or use a strategy for setting it
     let gas_price = client_arc.get_gas_price().await?;

    // Convert chain_id from String to U64
    let chain_id: U64 = network_metadata.chain_id.parse()?;
 
     // Create the TransactionRequest manually
     let tx_request = TransactionRequest {
        chain_id: Some(chain_id),
        from: Some(network_metadata.wallet_address.into()),
        to: Some(NameOrAddress::Address(router_contract_address)),
        gas: Some(estimated_gas),
        gas_price: Some(gas_price),
        value: Some(amount_in_wei), // This is the ETH amount you're sending
        data: Some(function_call.tx.data().unwrap().clone()), // Extract the data from the function call
        // Nonce is typically managed by the client, but you can specify it manually if needed
        nonce: None, // Set this to Some(nonce) if you're manually managing nonces
     };


    if network_metadata.chain_id == "1" {
        // Send the transaction with Flashbots
        send_tx_flashbots(
           network_metadata,
           tx_request
        ).await
    } else {
        // Send the transaction with the regular client
        send_tx(
           client_arc,
           tx_request
        ).await
    }
}

async fn calculate_amount_out_v2(
    router: &UniswapV2Router02<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    amount_in: U256,
    token_in: Address,
    token_out: Address,
) -> Result<U256, Box<dyn Error>> {
    // Define the path (token_in -> token_out)
    let path = vec![token_in, token_out];

    // Fetch the expected output amount
    let amounts_out: Vec<U256> = router.get_amounts_out(amount_in, path).call().await?;
    let amount_out = amounts_out.last().ok_or("Failed to get output amount")?;

    let amount_out_min = apply_slippage(amount_out, SLIPPAGE);
    
    Ok(amount_out_min)
}

async fn calculate_amount_out_v3(
    router: &UniswapV3SmartRouter<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    recipient: H160,
    amount_in: U256,
    token_in: Address,
    token_out: Address,
    fee: u32, // Fee tier, e.g., 3000 for 0.3%
) -> Result<U256, Box<dyn Error>> {
    let params = ExactInputSingleParams {
        token_in,
        token_out,
        fee,
        recipient,
        amount_in,
        amount_out_minimum: 0.into(), // Set to 0 for quoting
        sqrt_price_limit_x96: 0.into(), // Set to 0 to not set a specific price limit
    };

    let amount_out: U256 = router.exact_input_single(params).call().await?;
    
    // Convert the integer to U256
    let amount_out_min = apply_slippage(&amount_out, SLIPPAGE);
    
    Ok(amount_out_min)
}

fn apply_slippage(amount: &U256, slippage: f64) -> U256 {
    let slippage_factor = 1e18 as f64 * (1.0 - slippage); // Convert slippage to a factor with 18 decimal places
    let amount_with_slippage = amount.as_u128() as f64 * slippage_factor; // Apply slippage
    U256::from(amount_with_slippage as u128) // Convert back to U256
}