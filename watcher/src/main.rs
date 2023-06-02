use anyhow::Result;
use bitcoin::consensus::deserialize;
use inscription_parser::{Inscription, InscriptionError};
use lib::{Transaction, TransactionType};
use serde::{Deserialize, Serialize};
use tracing::log::info;
use std::str::FromStr;

use std::collections::HashSet;
use std::time::Duration;

use crate::inscription_parser::InscriptionParser;

mod abci_client;
mod inscription_parser;

const LOCAL_SEQUENCER_URL: &str = "http://127.0.0.1:26657";

// TODO: Handle errors better

#[derive(Serialize, Deserialize)]
struct OrdinalBody {
    tick: String,
    amt: String,
    op: String,
    p: String,
    starknet_address: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Change this for our vanity address
    let address = "1BitcoinEaterAddressDontSendf59kuE";
    let url = format!("https://blockchain.info/rawaddr/{}", address);

    let mut burned_transactions: HashSet<String> = HashSet::new();

    // let ins = get_ordinal_data("6e995548e5be3c6f215f9301ae0d53691100b23ddaa4e5b12076503d5b1646ca").await.unwrap();

    // println!("{} - {}", String::from_utf8(ins.content_type.unwrap()).unwrap(), String::from_utf8(ins.body.unwrap()).unwrap());

    loop {
        let response = reqwest::get(&url).await?.text().await?;
        let v: serde_json::Value = serde_json::from_str(&response)?;

        if let Some(transactions) = v["txs"].as_array() {
            for tx in transactions {
                let hash = tx["hash"]
                    .as_str()
                    .expect("Transaction does not contain hash");
                    
                let tx = Transaction::with_type(TransactionType::Mint {
                        address: "0x0".to_owned(),
                        amount: 10u64,
                        token_tick: "ordi".to_owned(),
                    });

                info!("Detected ordinal burned. Sending out transaction {:?} to Rollup", tx);

                let result = abci_client::broadcast(tx, LOCAL_SEQUENCER_URL).await;
                //if !burned_transactions.contains(hash) {
                //    println!("About to burn ERC-20 on Barknet: {}", hash);
//
                //    let inscription = match get_ordinal_data(hash).await {
                //        Err(_) => continue,
                //        Ok(v) => v,
                //    };
//
                //    let ord_body = match deserialize_validate_inscription_body(inscription) {
                //        Err(_) => continue,
                //        Ok(v) => v,
                //    };
//
                //    if ord_body.starknet_address.is_none() {
                //        println!("tx {} does not contain starknet address in metadata", hash);
                //        continue;
                //    }
//
                //    // call barknet and get status
                //    let starknet_address = ord_body.starknet_address.unwrap(); // TODO: Get address and tx data from bitcoin metadata
                //    let tx = Transaction::with_type(TransactionType::Mint {
                //        address: starknet_address.clone(),
                //        amount: u64::from_str(&ord_body.amt)?,
                //        token_tick: ord_body.tick,
                //    });
//
                //    let result = abci_client::broadcast(tx, LOCAL_SEQUENCER_URL).await;
//
                //    if result.is_ok() {
                //        println!(
                //            "Minted succesfully hash {} to Starknet address {}",
                //            hash, &starknet_address
                //        );
                //        burned_transactions.insert(hash.to_string());
                //    }
                //}
            }
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

async fn get_ordinal_data(tx_id: &str) -> Result<Inscription, InscriptionError> {
    let tx_hex_url = format!("https://blockchain.info/rawtx/{}?format=hex", tx_id);
    let tx_hex = reqwest::get(&tx_hex_url)
        .await
        .expect("Error with ordinal request")
        .text()
        .await
        .expect("Error parsing tx text");

    let tx_bytes = hex::decode(tx_hex).expect("Invalid hex");
    let btc_tx: bitcoin::Transaction = deserialize(&tx_bytes).expect("Invalid transaction");

    let inscription = InscriptionParser::parse(
        &btc_tx
            .input
            .first()
            .expect("Error getting input from tx")
            .witness,
    )?;

    Ok(inscription)
}

fn deserialize_validate_inscription_body(inscription: Inscription) -> Result<OrdinalBody> {
    // TODO: Check if the content type is text/plain;charset=utf-8
    // TODO: assert p=="brc-20"
    let inscription_body = inscription.body.expect("Inscription does not contain body");
    let inscription_body = String::from_utf8(inscription_body)?;

    let ord: OrdinalBody = serde_json::from_str(&inscription_body)?;

    Ok(ord)
}

#[cfg(test)]
mod tests {
    use crate::OrdinalBody;

    #[test]
    fn test_inscription_body_parsing() {
        let obj = r#"{"p":"brc-20","op":"mint","tick":"PDAY","amt":"500"}"#;
        let ord: OrdinalBody = serde_json::from_str(obj).unwrap();
        assert!(ord.starknet_address.is_none());

        let obj =
            r#"{"p":"brc-20","op":"mint","tick":"PDAY","amt":"500", "starknet_address": "0x0"}"#;
        let ord: OrdinalBody = serde_json::from_str(obj).unwrap();
        ord.starknet_address.unwrap();
    }
}
