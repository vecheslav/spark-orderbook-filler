use fuels::{
    accounts::wallet::WalletUnlocked,
    programs::calls::CallHandler,
    types::{transaction::TxPolicies, Bits256},
};
use spark_market_sdk::SparkMarketContract;
use std::{cmp, sync::Arc};
use tokio::sync::{Mutex, RwLock};

use crate::{
    orderbook::{OrderId, OrderType},
    types::{Amount, Asset},
};

#[derive(Debug, Clone)]
pub enum Operation {
    OpenOrder(OpenOrderOperation),
    CancelOrder(CancelOrderOperation),
}

#[derive(Debug, Clone)]
pub struct OpenOrderOperation {
    pub order_type: OrderType,
    pub base: Asset,
    pub quote: Asset,
    pub amount: Amount,
    pub price: u64,
}

#[derive(Debug, Clone)]
pub struct CancelOrderOperation {
    pub order_id: OrderId,
}

#[derive(Debug, Clone)]
pub struct OperationMessage {
    pub operation: Operation,
}

#[derive(Default)]
pub struct OperationManager {
    pub operations: Arc<Mutex<Vec<Operation>>>,
    pub multicall_size: usize,
}

impl OperationManager {
    pub fn new(multicall_size: usize) -> Self {
        Self {
            multicall_size,
            ..Self::default()
        }
    }

    pub async fn add(&self, message: &OperationMessage) -> usize {
        let OperationMessage { operation } = message;

        let mut operations = self.operations.lock().await;
        operations.push(operation.clone());

        operations.len()
    }

    pub async fn process(
        &self,
        trader: &WalletUnlocked,
        market_contract: &Arc<RwLock<SparkMarketContract>>,
    ) {
        let mut operations = self.operations.lock().await;
        let total_operations = operations.len();

        let bunch = operations
            .drain(..cmp::min(total_operations, self.multicall_size))
            .collect::<Vec<_>>();
        drop(operations);

        let mut multicall = CallHandler::new_multi_call(trader.clone());
        let market_contract = market_contract.read().await;

        multicall = bunch.iter().fold(multicall, |multicall, operation| {
            match operation.to_owned() {
                Operation::OpenOrder(OpenOrderOperation {
                    order_type,
                    base: _,
                    quote: _,
                    amount,
                    price,
                }) => {
                    let call = market_contract
                        .with_account(trader)
                        .get_instance()
                        .methods()
                        .open_order(*amount, order_type.into(), price);

                    multicall.add_call(call)
                }
                Operation::CancelOrder(CancelOrderOperation { order_id }) => {
                    let call = market_contract
                        .with_account(trader)
                        .get_instance()
                        .methods()
                        .cancel_order(Bits256::from_hex_str(&order_id).unwrap());

                    multicall.add_call(call)
                }
            }
        });
        drop(market_contract);

        // Send transactions without waiting for commit
        match multicall
            .with_tx_policies(
                TxPolicies::default()
                    .with_tip(1)
                    .with_script_gas_limit(800_000 * self.multicall_size as u64),
            )
            .submit()
            .await
        {
            Ok(res) => {
                log::info!("OK: {:?}", res.tx_id());
            }
            Err(e) => {
                log::error!("{:?}", e);
                // Revert bunch back to all calls
                let mut operations = self.operations.lock().await;
                operations.extend(bunch);
            }
        }
        // match multicall
        //     .with_tx_policies(
        //         TxPolicies::default()
        //             .with_tip(1)
        //             .with_script_gas_limit(800_000 * self.multicall_size as u64),
        //     )
        //     .call::<()>()
        //     .await
        // {
        //     Ok(CallResponse { tx_id, .. }) => {
        //         log::info!("OK: {:?}", tx_id);
        //     }
        //     Err(e) => {
        //         log::error!("{:?}", e);
        //         // Revert bunch back to all calls
        //         let mut operations = self.operations.lock().await;
        //         operations.extend(bunch);
        //     }
        // }
    }
}
