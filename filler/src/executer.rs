use fuels::{
    accounts::wallet::WalletUnlocked,
    programs::calls::{CallHandler, CallParameters, ContractCall},
    types::{transaction::TxPolicies, Bits256},
};
use spark_market_sdk::SparkMarketContract;
use std::{cmp, sync::Arc};
use tokio::sync::{Mutex, RwLock};

use crate::{
    orderbook::OrderType,
    types::{Amount, Asset},
};

// TODO: Only for orders now
#[derive(Debug, Clone)]
pub struct SignalMessage {
    pub order_type: OrderType,
    pub base: Asset,
    pub quote: Asset,
    pub amount: Amount,
    pub price: u64,
}

pub type Call = CallHandler<WalletUnlocked, ContractCall, Bits256>;

#[derive(Default)]
pub struct CallExecuter {
    pub calls: Arc<Mutex<Vec<Call>>>,
}

impl CallExecuter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the incoming signal message to open an order.
    ///
    /// This function extracts the necessary information from the `SignalMessage`
    /// and uses it to create and add a call to the `multi_call` for opening an order
    /// in the market contract.
    pub async fn handle_signal(
        &self,
        index: usize,
        message: SignalMessage,
        market_contract: Arc<RwLock<SparkMarketContract>>,
    ) {
        let SignalMessage {
            order_type,
            base: _,
            quote: _,
            amount,
            price,
        } = message;

        log::debug!(
            "{} / SIGNAL: {:?}, {}, {}",
            index,
            order_type,
            price,
            *amount
        );

        let market_contract = market_contract.read().await;
        let call =
            market_contract
                .get_instance()
                .methods()
                .open_order(*amount, order_type.into(), price);

        let mut calls = self.calls.lock().await;
        calls.push(call);
    }

    /// Submits the accumulated calls in the `multi_call`.
    ///
    /// This function reads the `multi_call`, and attempts to submit
    /// the accumulated calls. It logs the result of the submission.
    pub async fn submit(&self, index: usize, wallet: WalletUnlocked) {
        let mut calls = self.calls.lock().await;
        let total_calls = calls.len();
        let bunch_size = 15;

        if total_calls < bunch_size {
            return;
        }

        let bunch = calls
            .drain(..cmp::min(total_calls, bunch_size))
            .collect::<Vec<_>>();

        log::info!(
            "{} / BUNCH: {:?}, QUEUE: {:?}",
            index,
            bunch.len(),
            calls.len()
        );

        // Drop calls to avoid locking before submit
        drop(calls);

        let mut multi_call = CallHandler::new_multi_call(wallet.clone());
        for call in bunch.iter() {
            multi_call = multi_call.add_call(call.clone());
        }

        // Send transactions without waiting for commit
        match multi_call
            .with_tx_policies(
                TxPolicies::default()
                    .with_tip(1)
                    .with_script_gas_limit(800_000 * bunch_size as u64),
            )
            .submit()
            .await
        {
            Ok(res) => {
                log::info!("{} / OK: {:?}", index, res.tx_id());
            }
            Err(e) => {
                log::error!("{} / {:?}", index, e);
                // Revert bunch back to all calls
                let mut calls = self.calls.lock().await;
                calls.extend(bunch);
            }
        }
        // match multi_call
        //     .with_tx_policies(
        //         TxPolicies::default()
        //             .with_tip(1)
        //             // Aprox gas for 30 calls
        //             .with_script_gas_limit(20_000_000),
        //     )
        //     .call::<()>()
        //     .await
        // {
        //     Ok(CallResponse { tx_id, .. }) => {
        //         calls.clear();
        //         log::info!("{} / SUBMIT OK: {:?}", index, tx_id);
        //     }
        //     Err(e) => log::error!("{} / {:?}", index, e),
        // }
    }
}
