use std::sync::{Arc, Mutex};

use felt::Felt252;
use lib::{Transaction, TransactionType};
use sha2::{Digest, Sha256};
use starknet_contract_class::EntryPointType;
use starknet_rs::business_logic::execution::execution_entry_point::ExecutionEntryPoint;
use starknet_rs::business_logic::execution::{CallType, TransactionExecutionContext};
use starknet_rs::business_logic::fact_state::state::ExecutionResourcesManager;
use starknet_rs::business_logic::{
    fact_state::in_memory_state_reader::InMemoryStateReader, state::cached_state::CachedState,
};
use starknet_rs::core::contract_address::compute_deprecated_class_hash;
use starknet_rs::services::api::contract_classes::deprecated_contract_class::ContractClass;
use starknet_rs::utils::{calculate_sn_keccak, felt_to_hash};
use starknet_rs::{
    business_logic::{
        state::state_api::State, transaction::objects::internal_deploy::InternalDeploy,
    },
    definitions::general_config::StarknetGeneralConfig,
    utils::Address,
};
use std::path::PathBuf;
use tendermint_abci::Application;
use tendermint_proto::abci;

use tracing::{debug, info};

/// An Tendermint ABCI application that works with a Cairo backend.
/// This struct implements the ABCI application hooks, forwarding commands through
/// a channel for the parts that require knowledge of the application state and the Cairo details.
/// For reference see https://docs.tendermint.com/v0.34/introduction/what-is-tendermint.html#abci-overview
#[derive(Debug, Clone)]
pub struct StarknetApp {
    hasher: Arc<Mutex<Sha256>>,
    state: Arc<Mutex<CachedState<InMemoryStateReader>>>,
    general_config: StarknetGeneralConfig,
    #[allow(dead_code)]
    amm_contract_info: (Address, [u8; 32]),
    erc20_contract_info: (Address, [u8; 32]),
}

impl Application for StarknetApp {
    /// This hook is called once upon genesis. It's used to load a default set of records which
    /// make the initial distribution of credits in the system.
    fn init_chain(&self, _request: abci::RequestInitChain) -> abci::ResponseInitChain {
        info!("Loading genesis");

        Default::default()
    }

    /// This hook provides information about the ABCI application.
    fn info(&self, request: abci::RequestInfo) -> abci::ResponseInfo {
        debug!(
            "Got info request. Tendermint version: {}; Block version: {}; P2P version: {}",
            request.version, request.block_version, request.p2p_version
        );

        abci::ResponseInfo {
            data: "cairo-app".to_string(),
            version: "0.1.0".to_string(),
            app_version: 1,
            last_block_height: HeightFile::read_or_create(),

            // using a fixed hash, see the commit() hook
            last_block_app_hash: vec![],
        }
    }

    /// This hook is to query the application for data at the current or past height.
    fn query(&self, _request: abci::RequestQuery) -> abci::ResponseQuery {
        let query_result = Err("Query hook needs implementation");

        match query_result {
            Ok(value) => abci::ResponseQuery {
                value,
                ..Default::default()
            },
            Err(e) => abci::ResponseQuery {
                code: 1,
                log: format!("Error running query: {e}"),
                info: format!("Error running query: {e}"),
                ..Default::default()
            },
        }
    }

    /// This ABCI hook validates an incoming transaction before inserting it in the
    /// mempool and relaying it to other nodes.
    fn check_tx(&self, request: abci::RequestCheckTx) -> abci::ResponseCheckTx {
        info!("hex code for tx: {}", hex::encode(&request.tx));
        let tx: Transaction = bincode::deserialize(&request.tx).unwrap();

        match tx.transaction_type {
            TransactionType::Mint { .. } => {
                info!("Received mint transaction {:?}, minting", tx);
                let class_hash = self.erc20_contract_info.1;

                // create entry_point_selector for mint. It should be a Felt
                let entry_point_selector =
                    Felt252::from_bytes_be(&calculate_sn_keccak("mint".as_bytes()));
                let call_data = [Felt252::from(2), Felt252::from(10), Felt252::from(1)].to_vec();
                let address = Address(1.into());
                let contract_address = &self.erc20_contract_info.0;
                let entry_point_type = EntryPointType::External;

                let execution_entry_point = ExecutionEntryPoint::new(
                    contract_address.clone(),
                    call_data,
                    entry_point_selector,
                    address,
                    entry_point_type,
                    Some(CallType::Delegate),
                    Some(class_hash),
                    0,
                );

                let tx_execution_context = TransactionExecutionContext::new(
                    Address(1.into()),
                    Felt252::from(0),
                    Vec::new(),
                    0,
                    0.into(),
                    self.general_config.invoke_tx_max_n_steps(),
                    1,
                );

                let mut state = self.state.lock().unwrap();

                execution_entry_point
                    .execute(
                        &mut *state,
                        &self.general_config,
                        &mut ExecutionResourcesManager::default(),
                        &tx_execution_context,
                        false,
                    )
                    .unwrap();
            }
        }

        abci::ResponseCheckTx {
            ..Default::default()
        }
    }

    /// This hook is called before the app starts processing transactions on a block.
    /// Used to store current proposer and the previous block's voters to assign fees and coinbase
    /// credits when the block is committed.
    fn begin_block(&self, _request: abci::RequestBeginBlock) -> abci::ResponseBeginBlock {
        // because begin_block, [deliver_tx] and end_block/commit are on the same thread, this is safe to do (see declaration of statics)

        Default::default()
    }

    /// This ABCI hook validates a transaction and applies it to the application state,
    /// for example storing the program verifying keys upon a valid deployment.
    /// Here is also where transactions are indexed for querying the blockchain.
    fn deliver_tx(&self, request: abci::RequestDeliverTx) -> abci::ResponseDeliverTx {
        let tx: Transaction = bincode::deserialize(&request.tx).unwrap();

        // Validation consists of getting the hash and checking whether it is equal
        // to the tx id. The hash executes the program and hashes the trace.

        match tx.transaction_type {
            TransactionType::Mint { .. } => abci::ResponseDeliverTx {
                ..Default::default()
            },
        }
    }

    /// Applies validator set updates based on staking transactions included in the block.
    /// For details about validator set update semantics see:
    /// https://github.com/tendermint/tendermint/blob/v0.34.x/spec/abci/apps.md#endblock
    fn end_block(&self, _request: abci::RequestEndBlock) -> abci::ResponseEndBlock {
        abci::ResponseEndBlock {
            ..Default::default()
        }
    }

    /// This hook commits is called when the block is comitted (after deliver_tx has been called for each transaction).
    /// Changes to application should take effect here. Tendermint guarantees that no transaction is processed while this
    /// hook is running.
    /// The result includes a hash of the application state which will be included in the block header.
    /// This hash should be deterministic, different app state hashes will produce blockchain forks.
    /// New credits records are created to assign validator rewards.
    fn commit(&self) -> abci::ResponseCommit {
        // the app hash is intended to capture the state of the application that's not contained directly
        // in the blockchain transactions (as tendermint already accounts for that with other hashes).
        // https://github.com/tendermint/tendermint/issues/1179
        // https://github.com/tendermint/tendermint/blob/v0.34.x/spec/abci/apps.md#query-proofs

        let app_hash = self
            .hasher
            .lock()
            .map(|hasher| hasher.clone().finalize().as_slice().to_vec());

        let height = HeightFile::increment();

        info!("Committing height {}", height,);

        match app_hash {
            Ok(hash) => abci::ResponseCommit {
                data: hash,
                retain_height: 0,
            },
            // error should be handled here
            _ => abci::ResponseCommit {
                data: vec![],
                retain_height: 0,
            },
        }
    }
}

impl StarknetApp {
    /// Constructor.
    pub fn new() -> Self {
        let state_reader = InMemoryStateReader::default();
        let mut state = CachedState::new(state_reader, None, None);

        state.set_contract_classes(Default::default()).unwrap();

        let amm_contract_class =
            ContractClass::try_from(PathBuf::from("abci/starknet_programs/amm.json")).unwrap();
        let erc20_contract_class =
            ContractClass::try_from(PathBuf::from("abci/starknet_programs/ERC20Mintable.json"))
                .unwrap();

        let amm_class_hash =
            felt_to_hash(&compute_deprecated_class_hash(&amm_contract_class).unwrap());
        let erc20_class_hash =
            felt_to_hash(&compute_deprecated_class_hash(&erc20_contract_class).unwrap());

        state
            .set_contract_class(&amm_class_hash, &amm_contract_class)
            .unwrap();
        state
            .set_contract_class(&erc20_class_hash, &erc20_contract_class)
            .unwrap();

        let internal_deploy_amm = InternalDeploy::new(
            Address(1.into()),
            amm_contract_class.clone(),
            vec![],
            0.into(),
            0,
            None,
        )
        .unwrap();

        let internal_deploy_erc20 = InternalDeploy::new(
            Address(1.into()),
            erc20_contract_class.clone(),
            vec![
                1.into(),
                1.into(),
                1.into(),
                100.into(),
                1.into(),
                1.into(),
                1.into(),
            ],
            0.into(),
            0,
            None,
        )
        .unwrap();

        let general_config = StarknetGeneralConfig::default();

        let tx_execution_amm = internal_deploy_amm
            .apply(&mut state, &general_config)
            .unwrap();
        let tx_execution_erc20 = internal_deploy_erc20
            .apply(&mut state, &general_config)
            .unwrap();

        let amm_contract_info = (
            tx_execution_amm.call_info.unwrap().contract_address,
            amm_class_hash,
        );

        let erc20_contract_info = (
            tx_execution_erc20.call_info.unwrap().contract_address,
            erc20_class_hash,
        );

        let new_state = Self {
            hasher: Arc::new(Mutex::new(Sha256::new())),
            state: Arc::new(Mutex::new(state)),
            amm_contract_info,
            erc20_contract_info,
            general_config,
        };

        let height_file = HeightFile::read_or_create();

        info!(
            "Starting with Starknet State: {:?}. Height file has value: {}",
            new_state, height_file
        );

        new_state
    }
}

/// Local file used to track the last block height seen by the abci application.
struct HeightFile;

impl HeightFile {
    const PATH: &str = "abci.height";

    fn read_or_create() -> i64 {
        // if height file is missing or unreadable, create a new one from zero height
        if let Ok(bytes) = std::fs::read(Self::PATH) {
            // if contents are not readable, crash intentionally
            bincode::deserialize(&bytes).expect("Contents of height file are not readable")
        } else {
            std::fs::write(Self::PATH, bincode::serialize(&0i64).unwrap()).unwrap();
            0i64
        }
    }

    fn increment() -> i64 {
        // if the file is missing or contents are unexpected, we crash intentionally;
        let mut height: i64 = bincode::deserialize(&std::fs::read(Self::PATH).unwrap()).unwrap();
        height += 1;
        std::fs::write(Self::PATH, bincode::serialize(&height).unwrap()).unwrap();
        height
    }
}

// just covering a few special cases here. lower level test are done in record store and program store, higher level in integration tests.
#[cfg(test)]
mod tests {
    fn _test_hook() {}
}
