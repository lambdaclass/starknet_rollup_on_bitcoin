use lib::{Transaction, TransactionType};

use std::collections::HashSet;
use std::time::Duration;

mod abci_client;

const LOCAL_SEQUENCER_URL: &str = "http://127.0.0.1:26657";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Change this for our vanity address
    let address = "1BitcoinEaterAddressDontSendf59kuE";
    let url = format!("https://blockchain.info/rawaddr/{}", address);

    let mut burned_transactions: HashSet<String> = HashSet::new();

    loop {
        let response = reqwest::get(&url).await?.text().await?;
        let v: serde_json::Value = serde_json::from_str(&response)?;

        if let Some(transactions) = v["txs"].as_array() {
            for tx in transactions {
                let hash = tx["hash"]
                    .as_str()
                    .expect("Transaction does not contain hash")
                    .clone();
                if !burned_transactions.contains(hash) {
                    println!("About to burn ERC-20 on Barknet: {}", hash);
                    // call barknet and get status
                    let starknet_address = "0x0".to_string(); // TODO: Get address and tx data from bitcoin metadata
                    let tx = Transaction::with_type(TransactionType::Mint {
                        address: starknet_address.clone(),
                        amount: 100,
                        token_tick: "ordi".to_string(),
                    });

                    let result = abci_client::broadcast(tx, LOCAL_SEQUENCER_URL).await;

                    if result.is_ok() {
                        println!(
                            "Minted succesfully hash {} to Starknet address {}",
                            hash, &starknet_address
                        );
                        burned_transactions.insert(hash.to_string());
                    }
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
