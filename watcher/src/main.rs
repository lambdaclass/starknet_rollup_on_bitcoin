use std::{collections::HashSet};
use reqwest;
use serde_json;
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "1BitcoinEaterAddressDontSendf59kuE"; 
    let url = format!("https://blockchain.info/rawaddr/{}", address);

    let mut burned_transactions: HashSet<String> = HashSet::new();

    loop {
        let response = reqwest::get(&url).await?.text().await?;
        let v: serde_json::Value = serde_json::from_str(&response)?;

        if let Some(transactions) = v["txs"].as_array() {
            for tx in transactions {
                let hash = tx["hash"].as_str().expect("Transaction does not contain hash").clone();
                if !burned_transactions.contains(hash) {
                    println!("About to burn ERC-20 on Barknet: {}", hash);
                    // call barknet and get status
                    // if successful {
                        burned_transactions.insert(hash.to_string());
                    // }
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

