// TODO: This might have to be its own crate

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use tendermint_rpc::{HttpClient, Client};
use tracing::debug;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Transaction {
    // TODO: Add signature?
    pub transaction_type: TransactionType,
    pub id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum TransactionType {
    /// Execute a function from a deployed contract.
    Mint {
        address: String,
        amount: u64,
        token_tick: String,
    },
}

impl Transaction {
    pub fn with_type(transaction_type: TransactionType) -> Transaction {
        Transaction {
            transaction_type,
            id: Uuid::new_v4().to_string(),
        }
    }
}


pub async fn broadcast_async(transaction: Transaction, url: &str) -> Result<()> {
    let client = HttpClient::new(url).unwrap();
    let tx_bin = bincode::serialize(&transaction)?;
    let response = client.broadcast_tx_async(tx_bin.into()).await?;

    debug!("Response from CheckTx: {:?}", response);
    match response.code {
        tendermint::abci::Code::Ok => Ok(()),
        tendermint::abci::Code::Err(code) => {
            bail!("Error executing transaction {}: {}", code, response.log)
        }
    }
}