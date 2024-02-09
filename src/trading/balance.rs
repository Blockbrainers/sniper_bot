use std::sync::Arc;

use ethers::{types::{H160, U256}, providers::{Provider, Http, Middleware}};

use crate::{errors::SendableError, bindings::erc20::Erc20};

pub async fn get_token_balance(rpc_url: &str, wallet_address: H160, token_address: H160) -> Result<U256, SendableError> {
    // Attempt to create a provider and handle any errors that might occur.
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| SendableError::from(format!("Failed to create provider: {}", e)))?
        .into();

    // Create the contract object.
    let contract = Erc20::new(token_address, provider);

    // Attempt to fetch the balance and handle any errors that might occur.
    contract.balance_of(wallet_address).call().await
        .map_err(|e| SendableError::from(format!("Failed to fetch wallet balance: {}", e)))
}

pub async fn get_native_balance(rpc_url: &str, wallet_address: H160) -> Result<U256, SendableError> {
    // Create a provider connected to the Ethereum network with a type annotation
    let provider: Provider<Http> = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| SendableError::from(format!("Failed to create provider: {}", e)))?
        .into();

    // Fetch the native token balance of the bot's wallet address
    provider
        .get_balance(wallet_address, None).await
        .map_err(|e| SendableError::from(format!("Failed to fetch wallet balance: {}", e)))
}