use crate::{models::token_api_response_models::TokenSecurityDetails, utils::formatter::{parse_f64_field, parse_i32_field}};

const HIGH_RISK_COEFF: f64 = 5.0; // High-risk issues have a significant impact on the score.
const RISKY_COEFF: f64 = 3.0;     // Risky issues have a moderate impact.
const LOW_RISK_COEFF: f64 = 1.0;

pub fn calculate_security_score(token_info: &TokenSecurityDetails) -> f64 {
    let high_risk_score = apply_high_risk_checks(token_info) * HIGH_RISK_COEFF;

    println!("high_risk_score: {}", high_risk_score);
    if high_risk_score == 0.0 {
        return 0.0;
    }

    let risky_score = apply_risky_checks(token_info) * RISKY_COEFF;
    println!("risky_score: {}", risky_score);
    let low_risk_score = apply_low_risk_checks(token_info) * LOW_RISK_COEFF;
    println!("low_risk_score: {}", low_risk_score);
    // Calculate the final score as a weighted average or sum of the individual scores
    let final_score = (high_risk_score + risky_score + low_risk_score) / (HIGH_RISK_COEFF + RISKY_COEFF + LOW_RISK_COEFF);
    println!("final_score: {}", final_score);
    final_score.max(0.0).min(100.0) // Ensure it doesn't go below 0
}

fn apply_high_risk_checks(token_info: &TokenSecurityDetails) -> f64 {
    let mut high_risk_score: f64 = 100.0; // Start with a perfect score for high-risk checks

    // Check if the token is listed in any DEX.
    if token_info.is_in_dex.as_deref() != Some("1") {
        return 0.0; 
    }
    
    // Check if can buy and sell
    match token_info.cannot_buy.as_deref() {
        Some("1") => return 0.0, // Extremely risky, set high-risk score to 0 immediately.
        None => high_risk_score -= 30.0, // Heavy penalty for missing information.
        _ => (), // No change for other cases.
    }

    match token_info.cannot_sell_all.as_deref() {
        Some("1") => return 0.0, // Extremely risky, set high-risk score to 0 immediately.
        None => high_risk_score -= 30.0, // Heavy penalty for missing information.
        _ => (), // No change for other cases.
    }

    // Initialize liquidity as 0.0
    let mut liquidity = 0.0;

    // Process only if the dex list exists and contains exactly one element
    match &token_info.dex {
        Some(dex_list) if dex_list.len() == 1 => {
            let dex = &dex_list[0];  // Safely access the first and only dex entry

            if dex.liquidity.is_some() {
                // Attempt to parse the liquidity string to a float
                liquidity = dex.liquidity.as_ref().unwrap().parse::<f64>().unwrap_or(0.0);
            }
        },
        _ => {
            // Either dex doesn't exist or doesn't have exactly one entry
            return 0.0; 
        }
    }

    if liquidity == 0.0 {
        return 0.0;
    }

    // Honeypot Check
    match token_info.is_honeypot.as_deref() {
        Some("1") => return 0.0, // Extremely risky, set high-risk score to 0 immediately.
        None => high_risk_score -= 30.0, // Heavy penalty for missing information.
        _ => (), // No change for other cases.
    }

    // Same Creator as Known Honeypot Check
    if token_info.honeypot_with_same_creator.as_deref() == Some("1") {
        return 0.0; // Extremely risky if it has the same creator as a known honeypot.
    }

    // Self-Destruct Check
    if token_info.selfdestruct.as_deref() == Some("1") {
        return 0.0; // Self-destruct feature is a critical risk.
    }

    // Hidden Owner Check
    if token_info.hidden_owner.as_deref() == Some("1") {
        return 0.0; // Hidden owner suggests a lack of transparency and potential for malicious activity.
    }

    // Owner Can Change Balance Check
    if token_info.owner_change_balance.as_deref() == Some("1") {
        return 0.0; // If the owner can arbitrarily change balances, it's a significant risk.
    }

    // Proxy Contract Check
    if token_info.is_proxy.as_deref() == Some("1") {
        high_risk_score -= 50.0; // Proxy contracts add complexity and potential risks.
    }

    // Mintable Token Check
    if token_info.is_mintable.as_deref() == Some("1") {
        high_risk_score -= 50.0; // Mintable tokens can lead to inflation and devaluation.
    }

    // Ensure the high-risk score doesn't go below 0
    high_risk_score.max(0.0)
}

fn apply_risky_checks(token_info: &TokenSecurityDetails) -> f64 {
    let mut risky_score: f64 = 100.0; // Start with a perfect score for risky checks

    // Liquidity Check
    if let Some(dex_list) = &token_info.dex {
        if let Some(dex) = dex_list.get(0) {
            let liquidity = parse_f64_field(&dex.liquidity);
            risky_score -= if liquidity < 20000.0 { 20.0 } else { -5.0 };
        } else {
            risky_score -= 5.0; // Penalty for no DEX info
        }
    } else {
        risky_score -= 5.0; // Penalty for missing DEX info
    }

    // Buy Tax Check
    if let Some(_) = &token_info.buy_tax {
        let buy_tax = parse_f64_field(&token_info.buy_tax);  // Pass the entire Option<String>
        risky_score -= buy_tax * 100.0; // Decrease score based on buy tax percentage
    }

    // Sell Tax Check
    if let Some(_) = &token_info.sell_tax {
        let sell_tax = parse_f64_field(&token_info.sell_tax);  // Pass the entire Option<String>
        risky_score -= sell_tax * 100.0; // Decrease score based on sell tax percentage
    }

    // Anti Whale Mechanism Check
    match token_info.is_anti_whale.as_deref() {
        Some("1") => risky_score -= 10.0, // Presence of anti-whale mechanisms can be risky.
        None => risky_score -= 5.0,       // Penalty for missing information.
        _ => (),
    }

    // Modifiable Tax Rate Check
    match token_info.slippage_modifiable.as_deref() {
        Some("1") => risky_score -= 15.0, // Modifiable tax rates introduce unpredictability.
        None => risky_score -= 5.0,       // Penalty for missing information.
        _ => (),
    }

    // Owner Concentration Check
    if let Some(_) = &token_info.owner_percent {
        let owner_percent = parse_f64_field(&token_info.owner_percent);  // Pass the entire Option<String>
        risky_score -= if owner_percent > 50.0 { 30.0 } else { owner_percent / 2.0 }; // More concentration, higher penalty.
    }

    // Blacklisted Functionality Check
    match token_info.is_blacklisted.as_deref() {
        Some("1") => risky_score -= 20.0, // Blacklist functionality can be risky.
        None => risky_score -= 5.0,       // Penalty for missing information.
        _ => (),
    }

    // Transfer Pausable Check
    match token_info.transfer_pausable.as_deref() {
        Some("1") => risky_score -= 10.0, // Ability to pause transfers is risky.
        None => risky_score -= 5.0,       // Penalty for missing information.
        _ => (),
    }

    // Trading Cooldown Check
    match token_info.trading_cooldown.as_deref() {
        Some("1") => risky_score -= 10.0, // Trading cooldowns can be risky.
        None => risky_score -= 5.0,       // Penalty for missing information.
        _ => (),
    }

    // Ensure score doesn't go below 0
    risky_score.max(0.0).min(100.0)
}

fn apply_low_risk_checks(token_info: &TokenSecurityDetails) -> f64 {
    let mut low_risk_score: f64 = 100.0; // Start with a perfect low-risk score

    // Proxy Contract Check
    match token_info.is_proxy.as_deref() {
        Some("1") => low_risk_score -= 5.0, // Small penalty for being a proxy
        None => low_risk_score -= 2.0,      // Minor penalty for missing information
        _ => (),                            // No change for other cases.
    }

    // Open Source Check
    match token_info.is_open_source.as_deref() {
        Some("0") => low_risk_score -= 5.0, // Small penalty for closed source
        None => low_risk_score -= 2.0,      // Minor penalty for missing information
        _ => (),
    }

    // External Call Check
    match token_info.external_call.as_deref() {
        Some("1") => low_risk_score -= 5.0, // Small penalty for external calls
        None => low_risk_score -= 2.0,      // Minor penalty for missing information
        _ => (),
    }

    // Personal Slippage Modifiable Check
    match token_info.personal_slippage_modifiable.as_deref() {
        Some("1") => low_risk_score -= 5.0, // Small penalty for modifiable slippage
        None => low_risk_score -= 2.0,      // Minor penalty for missing information
        _ => (),
    }

    // Holder Count Check
    if let Some(holder_count_str) = &token_info.holder_count {
        let holder_count = holder_count_str.parse::<i32>().unwrap_or(0);
        low_risk_score -= if holder_count < 10 { 5.0 } else { 0.0 }; // Small penalty for very few holders
    } else {
        low_risk_score -= 2.0; // Minor penalty for missing information
    }

    // Whitelist Functionality Check
    match token_info.is_whitelisted.as_deref() {
        Some("1") => low_risk_score -= 5.0, // Small penalty for having a whitelist
        None => low_risk_score -= 2.0,      // Minor penalty for missing information
        _ => (),
    }

    // Anti Whale Modifiable Check
    match token_info.anti_whale_modifiable.as_deref() {
        Some("1") => low_risk_score -= 5.0, // Small penalty for modifiable anti-whale measures
        None => low_risk_score -= 2.0,      // Minor penalty for missing information
        _ => (),
    }

    // Top Holder Concentration Check
    if let Some(holders) = &token_info.holders {
        let top_holder_percent = holders.iter()
            .filter_map(|holder| holder.percent.as_ref().and_then(|p| p.parse::<f64>().ok()))
            .fold(0.0, |max, x| x.max(max)); // Find the max percentage
        
        low_risk_score -= if top_holder_percent > 50.0 { 5.0 } else { 0.0 }; // Small penalty if one holder has more than 50%
    }

    // Normalize the low-risk score to a scale of 0-100 and ensure it doesn't go below 0
    low_risk_score.max(0.0).min(100.0)
}