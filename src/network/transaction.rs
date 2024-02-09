use std::{sync::Arc, error::Error};

use ethers::{providers::{Provider, Http, Middleware}, signers::LocalWallet, types::{H256, TransactionRequest}, middleware::SignerMiddleware};

use crate::models::config_models::NetworkMetadata;

use super::client::create_flashbot_client;

pub async fn send_tx(
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    tx_request: TransactionRequest,
) -> Result<H256, Box<dyn Error>> {
    // Attempt to send the transaction
    // let pending_tx = match client.send_transaction(tx_request, None).await {
    //     Ok(tx) => tx,
    //     Err(e) => return Err(Box::new(e) as Box<dyn Error>),
    // };


    let pending_tx = match client.send_transaction(tx_request, None).await {
        Ok(tx) => {
            println!("************** Transaction Status: {}:", tx.tx_hash());
            tx
        }
        Err(e) => {
            println!("************** Error waiting for Tansaction to be mined: {:?}", e);
            return Err(Box::new(e) as Box<dyn Error>);
        }
    };

    // Await the transaction to be mined
    let receipt = match pending_tx.await {
        Ok(receipt) => {
            println!("Transaction mined. Status");
            receipt
        }
        Err(e) => {
            println!("Error waiting for transaction to be mined: {:?}", e);
            return Err(Box::new(e) as Box<dyn Error>);
        }
    };
    // Ensure the receipt is available
    let receipt = receipt.ok_or_else(|| {
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Transaction receipt not found")) as Box<dyn Error>
    })?;

    // Extract the transaction hash
    let transaction_hash = receipt.transaction_hash;

    // Return the transaction hash
    Ok(transaction_hash)
}

pub async fn send_tx_flashbots(
    network_metadata: &NetworkMetadata,
    tx_request: TransactionRequest,
) -> Result<H256, Box<dyn Error>> {

    // Create a Flashbots client
    let flashbots_client = create_flashbot_client(network_metadata.rpc_url.as_str()).await?;
    
    // Send the transaction using the Flashbots middleware
    let pending_tx = flashbots_client.send_transaction(tx_request, None).await?;

    // Await the transaction to be mined and get the receipt
    let receipt = pending_tx
        .await?
        .ok_or_else(|| Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Transaction not included",
        )))?;

    // Extract the transaction hash
    let tx_hash: H256 = receipt.transaction_hash;

    // Return the transaction hash
    Ok::<H256, Box<dyn Error>>(tx_hash)
}