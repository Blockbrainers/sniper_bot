use std::error::Error;

use ethers::types::U256;
use serde_json::Value;

use crate::{models::config_models::NetworkMetadata, errors::SendableError, trading::balance::{get_token_balance, get_native_balance}};

const MIN_WALLET_BALANCE_THRESHOLD: f64 = 0.1; // Minimum balance in wallet to consider trading (e.g., 0.1 wrapped tokens)
const MIN_LIQUIDITY_THRESHOLD: f64 = 1000.0; // Minimum liquidity in the pool to consider trading
const MIN_CONFIDENCE_THRESHOLD: f64 = 50.0; // Minimum confidence score to proceed with the trade
const MIN_TRADE_AMOUNT: f64 = 0.0; // Minimum amount to trade, could be set to a small number instead of 0

pub async fn calculate_amount_in(
    network_metadata: &NetworkMetadata,
    confidence_score: f64, 
    liquidity: f64
) -> Result<f64, SendableError> {
    // Safety checks
    if liquidity < MIN_LIQUIDITY_THRESHOLD || confidence_score < MIN_CONFIDENCE_THRESHOLD {
        return Ok(MIN_TRADE_AMOUNT); // Not enough liquidity or confidence, return minimum trade amount
    }
    let max_wallet_trade_percentage = 0.50; // 50% of wallet balance
    let max_price_impact = 0.01; // 1% maximum price impact

    // Correctly await the async function and handle the Result
    let max_safe_trade_amount = calculate_max_safe_trade_amount(liquidity, max_price_impact, &network_metadata.native_coin_coingecko_id).await
        .map_err(|_| SendableError::from("Error retrieving max_safe_trade_amount"))?;

    let score_adjustment_factor = confidence_score / 100.0; // Adjusting confidence score to a 0-1 range

    // Fetch wallet balance
    let wallet_balance_wei = bot_native_token_balance(network_metadata).await?;
    // let wallet_balance_wei = U256::from(500000000000000000u128);
    // Convert the balance to ETH for easier calculation
    let eth_precision = U256::exp10(18); // Represents 10^18 for conversion
    let wallet_balance_eth = wallet_balance_wei.as_u128() as f64 / eth_precision.as_u128() as f64;
    // Ensure wallet balance is above the minimum threshold
    if wallet_balance_eth < MIN_WALLET_BALANCE_THRESHOLD {
        return Ok(MIN_TRADE_AMOUNT); // Not enough balance, return minimum trade amount
    }

    // Calculate the maximum trade amount based on wallet balance
    let max_wallet_trade_amount = wallet_balance_eth * max_wallet_trade_percentage;

    // Calculate preliminary trade amount considering the confidence score and safe trade limit
    let preliminary_trade_amount = max_safe_trade_amount * score_adjustment_factor;

    // Final trade amount is the minimum of all the calculated limits
    let final_trade_amount = preliminary_trade_amount
        .min(max_wallet_trade_amount)
        .min(wallet_balance_eth);

    Ok(final_trade_amount)
}

async fn calculate_max_safe_trade_amount(
    liquidity_in_usdt: f64, // Liquidity of the pool in USDT
    max_price_impact: f64,   // Desired maximum price impact (e.g., 0.01 for 1%)
    native_coin_coingecko_id: &str
) -> Result<f64, Box<dyn Error>> {
    // Await the future to get the actual f64 value
    let eth_price_in_usdt = fetch_eth_price_in_usdt(native_coin_coingecko_id).await?;

    let liquidity_in_eth = liquidity_in_usdt / eth_price_in_usdt;

    let adjusted_max_price_impact = max_price_impact.clamp(0.0, 1.0);
    // Now you can perform the multiplication since you have the actual f64 values
    let max_safe_trade_amount_in_eth = liquidity_in_eth * adjusted_max_price_impact;

    Ok(max_safe_trade_amount_in_eth)
}

// async fn bot_wrapped_token_balance(network_metadata: &NetworkMetadata) -> Result<U256, SendableError> {
//     get_token_balance(&network_metadata.rpc_url, network_metadata.wallet_address, network_metadata.wrapped_native_address).await
//         .map_err(|e| SendableError::from(format!("Failed to fetch wallet balance: {}", e)))
// }

async fn bot_native_token_balance(network_metadata: &NetworkMetadata) -> Result<U256, SendableError> {
    get_native_balance(&network_metadata.rpc_url, network_metadata.wallet_address).await
        .map_err(|e| SendableError::from(format!("Failed to fetch wallet balance: {}", e)))
}

async fn fetch_eth_price_in_usdt(native_coin_coingecko_id: &str) -> Result<f64, Box<dyn Error>> {
    // Define the URL for the CoinGecko API request
    let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd", native_coin_coingecko_id);
    // Send a GET request to the URL and await the text response
    let resp = reqwest::get(url).await?.text().await?;
    // Parse the JSON response text into a serde_json Value
    let json: Value = serde_json::from_str(&resp)?;
    // Attempt to extract and return the Ethereum price in USDT
    let eth_price_in_usdt = json[native_coin_coingecko_id]["usd"]
        .as_f64()
        .ok_or_else(|| "Unable to find Ethereum price in USD")?;
    Ok(eth_price_in_usdt)
}
