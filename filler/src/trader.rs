use fuels::{
    accounts::wallet::WalletUnlocked,
    types::{ContractId, Identity},
};
use spark_market_sdk::SparkMarketContract;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc::unbounded_channel, Mutex, RwLock},
    time,
};

use crate::{
    executer::{CallExecuter, SignalMessage},
    types::{Receiver, Sender},
};

pub struct Trader {
    pub index: usize,
    pub wallet: WalletUnlocked,
    pub market_contract: Arc<RwLock<SparkMarketContract>>,
    pub executer: Arc<CallExecuter>,
    pub signal_tx: Sender<SignalMessage>,
    pub signal_rx: Receiver<SignalMessage>,
}

impl Trader {
    pub async fn new(index: usize, market_id: ContractId, wallet: WalletUnlocked) -> Self {
        let market_contract = SparkMarketContract::new(market_id, wallet.clone()).await;

        let (signal_tx, signal_rx) = unbounded_channel::<SignalMessage>();
        let executer = CallExecuter::new();

        Self {
            index,
            wallet,
            market_contract: Arc::new(RwLock::new(market_contract)),
            executer: Arc::new(executer),
            signal_tx: Arc::new(signal_tx),
            signal_rx: Arc::new(Mutex::new(signal_rx)),
        }
    }

    pub async fn run(&self) {
        let identity: Identity = self.wallet.address().into();
        log::info!("{} / IDENTITY: {:?}", self.index, identity);

        self.start_handle_signals().await;
        self.start_executer().await;
    }

    /// Start handle signals
    pub async fn start_handle_signals(&self) {
        let index = self.index;
        let executer = self.executer.clone();
        let signal_rx = self.signal_rx.clone();
        let market_contract = self.market_contract.clone();

        // Start handle signals
        tokio::spawn(async move {
            while let Some(signal) = signal_rx.lock().await.recv().await {
                executer
                    .handle_signal(index, signal, market_contract.clone())
                    .await;
            }
        });
    }

    /// Start the executer to submit the accumulated calls
    pub async fn start_executer(&self) {
        let index = self.index;
        let executer = self.executer.clone();
        let wallet = self.wallet.clone();

        tokio::spawn(async move {
            loop {
                time::sleep(Duration::from_millis(400)).await;

                executer.submit(index, wallet.clone()).await;
            }
        });
    }
}
