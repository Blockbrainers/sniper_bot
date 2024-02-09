use std::{error::Error};

use crate::{
    models::config_models::{NetworkMetadata, ExchangeConfig},
    bindings::{uniswap_v2_router02::UniswapV2Router02, uniswap_v3_smart_router::{UniswapV3SmartRouter, ExactInputSingleParams}},
    network::{client::{create_client_arc}, transaction::{send_tx, send_tx_flashbots}}
};

use ethers::{
    prelude::*,
    types::{Address, U256},
    utils::parse_ether, providers::Middleware,
};
use chrono::Utc;

pub async fn sell_token(
    exchange: ExchangeConfig,
    network_metadata: NetworkMetadata,
    target_token_address: Address,
    amount_in_tokens: U256,
    amount_out_min: U256,
    is_v3: bool,
    fee: Option<u32>,
) -> Result<H256, Box<dyn Error>> {
    if is_v3 {
        sell_v3(
            exchange,
            network_metadata,
            target_token_address,
            amount_in_tokens,
            amount_out_min,
            fee,
        )
        .await
    } else {
        sell_v2(
            exchange,
            network_metadata,
            target_token_address,
            amount_in_tokens,
            amount_out_min,
        )
        .await
    }
}

async fn sell_v2(
    exchange: ExchangeConfig,
    network_metadata: NetworkMetadata,
    target_token_address: Address,
    amount_in_tokens: U256,
    amount_out_min: U256,
) -> Result<H256, Box<dyn Error>> {
    let client_arc = create_client_arc(network_metadata.rpc_url.as_str(), network_metadata.clone().chain_id).await?;
    let router_contract_address: Address = exchange.router_contract_address.parse()?;
    let uniswap_v2_router = UniswapV2Router02::new(router_contract_address, client_arc.clone());

    let deadline = Utc::now().timestamp() as u64 + 15 * 60;
    let deadline_u256: U256 = U256::from(deadline);

    let path: Vec<Address> = vec![target_token_address, network_metadata.wrapped_native_address];

    let function_call = uniswap_v2_router.swap_exact_tokens_for_eth(
        amount_in_tokens,
        amount_out_min,
        path,
        network_metadata.wallet_address,
        deadline_u256,
    );

    let estimated_gas = function_call.estimate_gas().await?;
    let gas_price = client_arc.get_gas_price().await?;
    let chain_id: U64 = network_metadata.chain_id.parse()?;

    let tx_request = TransactionRequest {
        chain_id: Some(chain_id),
        from: Some(network_metadata.wallet_address.into()),
        to: Some(NameOrAddress::Address(router_contract_address)),
        gas: Some(estimated_gas),
        gas_price: Some(gas_price),
        value: None,  // No ETH sent along with token swap
        data: Some(function_call.tx.data().unwrap().clone()),
        nonce: None,
    };

    if network_metadata.chain_id == "1" {
        send_tx_flashbots(
            &network_metadata,
            tx_request,
        ).await
    } else {
        send_tx(
            client_arc,
            tx_request,
        ).await
    }
}

async fn sell_v3(
    exchange: ExchangeConfig,
    network_metadata: NetworkMetadata,
    target_token_address: Address,
    amount_in_tokens: U256,
    amount_out_min: U256,
    fee: Option<u32>,
) -> Result<H256, Box<dyn Error>> {

    let client_arc = create_client_arc(network_metadata.rpc_url.as_str(), network_metadata.chain_id.clone()).await?;
    let router_contract_address: Address = exchange.router_contract_address.parse()?;
    let uniswap_v3_router = UniswapV3SmartRouter::new(router_contract_address, client_arc.clone());

    let default_fee = 3000; 
    let fee = fee.unwrap_or(default_fee);

    let params = ExactInputSingleParams {
        token_in: target_token_address,
        token_out: network_metadata.wrapped_native_address,
        fee,
        recipient: network_metadata.wallet_address,
        amount_in: amount_in_tokens,
        amount_out_minimum: amount_out_min,
        sqrt_price_limit_x96: U256::zero(),
    };

    let function_call = uniswap_v3_router.exact_input_single(params);
    let estimated_gas = function_call.estimate_gas().await?;
    let gas_price = client_arc.get_gas_price().await?;
    let chain_id: U64 = network_metadata.chain_id.parse()?;

    let tx_request = TransactionRequest {
        chain_id: Some(chain_id),
        from: Some(network_metadata.wallet_address.into()),
        to: Some(NameOrAddress::Address(router_contract_address)),
        gas: Some(estimated_gas),
        gas_price: Some(gas_price),
        value: None,  // No ETH sent along with token swap
        data: Some(function_call.tx.data().unwrap().clone()),
        nonce: None,
    };

    if network_metadata.chain_id == "1" {
        send_tx_flashbots(
           &network_metadata,
           tx_request
        ).await
    } else {
        send_tx(
           client_arc,
           tx_request
        ).await
    }
}