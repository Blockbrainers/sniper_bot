use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TokenSecurityResponse {
    // code: i32,
    // message: String,
    pub result: std::collections::HashMap<String, TokenSecurityDetails>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TokenSecurityDetails {
    pub anti_whale_modifiable: Option<String>,  // "0" or "1" as string
    pub buy_tax: Option<String>,                   // Decimal as String
    pub can_take_back_ownership: Option<String>,// "0" or "1" as string
    pub cannot_buy: Option<String>,             // "0" or "1" as string
    pub cannot_sell_all: Option<String>,        // "0" or "1" as string
    pub creator_address: Option<String>,        // Address as string
    pub creator_balance: Option<String>,           // Decimal as String
    pub creator_percent: Option<String>,           // Decimal as String
    pub dex: Option<Vec<DexInfo>>,              // Array of DexInfo
    pub external_call: Option<String>,          // "0" or "1" as string
    pub hidden_owner: Option<String>,           // "0" or "1" as string
    pub holder_count: Option<String>,              // Integer as i32
    pub holders: Option<Vec<HolderInfo>>,               // Array of HolderInfo
    pub is_anti_whale: Option<String>,          // "0" or "1" as string
    pub is_blacklisted: Option<String>,         // "0" or "1" as string
    pub is_honeypot: Option<String>,            // "0" or "1" as string
    pub is_in_dex: Option<String>,              // "0" or "1" as string
    pub honeypot_with_same_creator: Option<String>,   // "0" or "1" as string
    pub is_mintable: Option<String>,            // "0" or "1" as string
    pub is_open_source: Option<String>,         // "0" or "1" as string
    pub is_proxy: Option<String>,               // "0" or "1" as string
    pub is_whitelisted: Option<String>,         // "0" or "1" as string
    pub lp_holder_count: Option<String>,           // Integer as i32
    pub owner_address: Option<String>,          // Address as string
    pub owner_balance: Option<String>,             // Decimal as String
    pub owner_change_balance: Option<String>,   // "0" or "1" as string
    pub owner_percent: Option<String>,             // Decimal as String
    pub personal_slippage_modifiable: Option<String>, // "0" or "1" as string
    pub selfdestruct: Option<String>,           // "0" or "1" as string
    pub sell_tax: Option<String>,                  // Decimal as String
    pub slippage_modifiable: Option<String>,    // "0" or "1" as string
    pub token_name: Option<String>,             // String
    pub token_symbol: Option<String>,           // String
    pub total_supply: Option<String>,              // Decimal as String
    pub trading_cooldown: Option<String>,       // "0" or "1" as string
    pub transfer_pausable: Option<String>,      // "0" or "1" as string
}

#[derive(Deserialize, Debug, Clone)]
pub struct DexInfo {
    pub liquidity_type: Option<String>, // String
    pub name: Option<String>,           // String
    pub liquidity: Option<String>,         // Decimal as String
    pub pair: Option<String>,           // Address as string
}

#[derive(Deserialize, Debug, Clone)]
pub struct HolderInfo {
    pub address: Option<String>,  // Address as string
    pub tag: Option<String>,      // String
    pub is_contract: Option<i32>, // "0" or "1" as integer
    pub balance: Option<String>,     // Decimal as String
    pub percent: Option<String>,     // Decimal as String
    pub is_locked: Option<i32>,   // "0" or "1" as integer
}

#[derive(Deserialize, Debug)]
pub struct LPHolderInfo {
    pub address: Option<String>,
    pub locked: Option<String>,
    pub tag: Option<String>,
    pub is_contract: Option<String>,
    pub balance: Option<String>,
    pub percent: Option<String>,
    pub NFT_list: Vec<NFTDetail>,
    pub locked_detail: Vec<LockDetail>,
}

#[derive(Deserialize, Debug)]
pub struct LockDetail {
    pub amount: Option<String>,
    pub end_time: Option<String>,
    pub opt_time: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct NFTDetail {
    pub value: Option<String>,
    pub NFT_id: Option<String>,
    pub amount: Option<String>,
    pub in_effect: Option<String>,
    pub NFT_percentage: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct FakeTokenInfo {
    pub true_token_address: Option<String>,
    pub value: Option<String>,
}