use std::{sync::Arc, error::Error};
use std::env;
use std::str::FromStr;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::signers::{LocalWallet, Signer};
use ethers::{providers::{Provider, Http}, signers::Wallet, middleware::SignerMiddleware};
use ethers_flashbots::FlashbotsMiddleware;
use reqwest::Url;

pub async fn create_client_arc(rpc_url: &str, chain_id: String) -> Result<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>, Box<dyn Error>> {
    let private_key = env::var("PRIVATE_KEY")?;
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let wallet = Wallet::from_str(&private_key)?.with_chain_id(chain_id.parse::<u64>()?);
    let client = SignerMiddleware::new(provider, wallet);
    Ok(Arc::new(client))
}

pub async fn create_flashbot_client(rpc_url: &str) -> Result<Arc<SignerMiddleware<FlashbotsMiddleware<Provider<Http>, Wallet<SigningKey>>, Wallet<SigningKey>>>, Box<dyn Error>> {
    let private_key = env::var("PRIVATE_KEY")?;
    let provider = Provider::<Http>::try_from(rpc_url)?;
    // This is your searcher identity
    let wallet = LocalWallet::from_str(&private_key)?;
    let bundle_signer = LocalWallet::from_str(&private_key)?;
    let client = SignerMiddleware::new(
        FlashbotsMiddleware::new(
            provider,
            Url::parse("https://relay.flashbots.net")?,
            bundle_signer,
        ),
        wallet,
    );
    
    Ok(Arc::new(client))
}