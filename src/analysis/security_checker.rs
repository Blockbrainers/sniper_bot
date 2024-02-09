use std::time::Duration;
use ethers::types::H160;
use tokio::time::sleep;

use crate::{
    models::{config_models::NetworkMetadata, token_api_response_models::{TokenSecurityDetails, TokenSecurityResponse}, security_models::TokenAssessment},
    errors::SendableError,
};

use super::{score_calculator::calculate_security_score, amount_in_calculator::calculate_amount_in};

pub async fn assess_token_security(network_metadata: &NetworkMetadata, exchange_name: &str, token_address: &H160) -> Result<TokenAssessment, SendableError> {
    const MAX_RETRIES: usize = 3;
    const BASE_BACKOFF: u64 = 5; // seconds

    for attempt in 0..MAX_RETRIES {
        match fetch_and_assess_token(network_metadata, exchange_name, token_address).await {
            Ok(assessment) 
            if assessment.confidence_score >= 70.0 => {
                // If the confidence score is acceptable, return the assessment immediately
                println!("Acceptable confidence score of {} achieved, breaking loop.", assessment.confidence_score);
                return Ok(assessment);
            }
            Ok(_) | Err(_) if attempt < MAX_RETRIES - 1 => {
                // If the confidence score is too low or an error occurred, retry after a delay
                let backoff = BASE_BACKOFF * 2_u64.pow(attempt as u32); // Exponential backoff
                println!("Retrying token assessment due to low confidence score or error. Attempt {} of {}", attempt + 1, MAX_RETRIES);
                sleep(Duration::from_secs(backoff)).await;
            }
            _ => {
                // If we've reached the maximum number of retries or got an unexpected result, break and return the last result.
                println!("Maximum retries exceeded or an unexpected result received. Returning the last result.");
                break;
            }
        }
    }

    // Last attempt after retries exhausted.
    fetch_and_assess_token(network_metadata, exchange_name, token_address).await
}

async fn fetch_and_assess_token(network_metadata: &NetworkMetadata, exchange_name: &str, token_address: &H160) -> Result<TokenAssessment, SendableError> {

    let token_info = fetch_token_security_info(&network_metadata.chain_id, token_address).await.map_err(SendableError::from)?;
    
    // Calculate the security score based on various factors
    let confidence_score = calculate_security_score(&token_info);

    let mut liquidity = 0.0;

    // Process only if the dex list exists and contains exactly one element
    match &token_info.dex {
        Some(dex_list) if dex_list.len() == 1 => {
            let dex = &dex_list[0];  // Safely access the first and only dex entry

            if let Some(dex_name) = &dex.name {
                if dex_name == exchange_name && dex.liquidity.is_some() {
                    // Attempt to parse the liquidity string to a float
                    liquidity = dex.liquidity.as_ref().unwrap().parse::<f64>().unwrap_or(0.0);
                }
            }
        },
        _ => {
            // Either dex doesn't exist or doesn't have exactly one entry
            // Consider setting a lower confidence score because of missing or multiple DEX entries
            return Ok(TokenAssessment {
                confidence_score: 0.0, // or some other value that indicates reduced confidence
                recommended_trade_amount: 0.0,
            });
        }
    }

    if liquidity == 0.0 {
        return Ok(TokenAssessment {
            confidence_score: 0.0, // or some other value that indicates reduced confidence
            recommended_trade_amount: 0.0,
        });
    }

    
    // Now, we need to await the result of calculate_amount_in since it's async
    let recommended_trade_amount = calculate_amount_in(network_metadata, confidence_score, liquidity).await?;
    
    // Return the assessment
    Ok(TokenAssessment {
        confidence_score,
        recommended_trade_amount, // Replace with actual calculation
    })
}

async fn fetch_token_security_info(network_chain_id: &str, token_address: &H160) -> Result<TokenSecurityDetails, SendableError> {
    let token_address_str = format!("{:#x}", token_address);
    let api_url = format!("https://api.gopluslabs.io/api/v1/token_security/{}?contract_addresses={}", network_chain_id, token_address_str);

    // let response_text = reqwest::get(&api_url).await?.text().await?;
    // let api_response: TokenSecurityResponse = serde_json::from_str(&response_text)?;
    let response_text = reqwest::get(&api_url).await.map_err(SendableError::new)?.text().await.map_err(SendableError::new)?;
    let api_response: TokenSecurityResponse = serde_json::from_str(&response_text).map_err(SendableError::new)?;

    // Extract TokenSecurityDetails from the response
    if let Some(details) = api_response.result.get(&token_address_str) {
        Ok(details.clone())  // Clone the details to match the expected type
    } else {
        Err("Token details not found in the response.".into())
    }
}