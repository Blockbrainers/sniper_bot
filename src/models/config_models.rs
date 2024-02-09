use serde::Deserialize;
use ethers::types::H160;

#[derive(Deserialize)]
pub struct Config {
    pub networks: Vec<NetworkConfig>,
}

#[derive(Deserialize)]
pub struct NetworkConfig {
    pub metadata: NetworkMetadata,
    pub exchanges: Vec<ExchangeConfig>,
}

#[derive(Deserialize, Clone)]
pub struct NetworkMetadata {
    pub name: String,
    pub symbol: String,
    #[serde(rename = "chainId")]
    pub chain_id: String,
    #[serde(rename = "nativeCoinCoingeckoId")]
    pub native_coin_coingecko_id: String,
    #[serde(rename = "rpcUrl")]
    pub rpc_url: String,
    #[serde(rename = "wsUrl")]
    pub ws_url: String,
    #[serde(rename = "walletAddress")]
    pub wallet_address: H160,
    #[serde(rename = "wrappedNativeAddress")]
    pub wrapped_native_address: H160,
}

#[derive(Deserialize, Clone)]
pub struct ExchangeConfig {
    pub name: String,
    #[serde(rename = "baseImplementation")]
    pub base_implementation: String,
    #[serde(rename = "factoryContractAddress")]
    pub factory_contract_address: String,
    #[serde(rename = "routerContractAddress")]
    pub router_contract_address: String,
}