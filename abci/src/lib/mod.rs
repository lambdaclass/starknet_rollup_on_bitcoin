// TODO: This might have to be its own crate

use serde::{Deserialize, Serialize};
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
