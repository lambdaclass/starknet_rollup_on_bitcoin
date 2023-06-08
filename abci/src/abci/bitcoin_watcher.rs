use anyhow::Result;
use lib::{Transaction, TransactionType};
use tendermint::node::info;
use tokio::runtime::Runtime;
use std::{sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender}, thread};
use tracing::info;

#[derive(Clone, Debug)]
pub struct BitcoinWatcher {
    /// Channel used to send operations to the task that manages the store state.
    command_sender: Sender<Command>,
}

pub enum Command {
    CheckBitcoin {
        block_number: u128
    }
}

impl BitcoinWatcher {
    pub fn new() -> Self {
        let (command_sender, command_receiver): (Sender<Command>, Receiver<Command>) = channel();

        thread::spawn(move ||  {
            let rt = Runtime::new().unwrap();

            while let Ok(command) = command_receiver.recv() {
                match command {
                    Command::CheckBitcoin {block_number } => {
                        // we would have to go ahead and check for new transactions on bitcoin here
                        
                        info!("Going to bitcoin and checking burn txs");
                        let tx = Transaction::with_type(TransactionType::Mint { address: "0x1".to_string(), amount: block_number.try_into().unwrap(), token_tick: "ordi".to_string() });
            
                        let _out = rt.block_on(lib::broadcast_async(tx, "http://127.0.0.1:26657"));
                    }
                };
            }
        });


        let bitcoin_watcher = Self { command_sender };

        bitcoin_watcher
    }

    pub fn check_and_mint(&self, bitcoin_block: u128) -> Result<()> {
        self.command_sender.send(Command::CheckBitcoin { block_number: bitcoin_block })?;
        Ok(())
    }
}