use std::collections::HashMap;

use anyhow::{ensure, Result};
use felt::Felt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub id: String,
    pub transaction_hash: String, // this acts
    pub transaction_type: TransactionType,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum TransactionType {
    /// Create new contract class.
    Declare,

    /// Create an instance of a contract which will have storage assigned. (Accounts are a contract themselves)
    Deploy,

    /// Execute a function from a deployed contract.
    Invoke,

    // TODO: Remove this when other transactions are implemented
    FunctionExecution {
        function: String,
        program_name: String,
    },
}

impl Transaction {
    pub fn with_type(transaction_type: TransactionType) -> Result<Transaction> {
        Ok(Transaction {
            transaction_hash: transaction_type.compute_and_hash()?,
            transaction_type,
            id: Uuid::new_v4().to_string(),
        })
    }

    /// Verify that the transaction id is consistent with its contents, by checking its sha256 hash.
    pub fn verify(&self) -> Result<()> {
        ensure!(
            self.transaction_hash == self.transaction_type.compute_and_hash()?,
            "Corrupted transaction: Inconsistent transaction id"
        );

        Ok(())
    }
}

impl TransactionType {
    pub fn compute_and_hash(&self) -> Result<String> {
        match self {
            TransactionType::FunctionExecution {
                function: _,
                program_name: _,
            } => todo!(),
            TransactionType::Declare => todo!(),
            TransactionType::Deploy => todo!(),
            TransactionType::Invoke => todo!(),
        }
    }
}
