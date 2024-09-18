use fuels::types::AssetId;
use std::{ops::Deref, sync::Arc};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

#[derive(Debug, Clone)]
pub struct Asset {
    pub id: AssetId,
    pub decimals: u8,
}

impl Asset {
    pub fn new(id: AssetId, decimals: u8) -> Self {
        Self { id, decimals }
    }
}

#[derive(Debug, Clone)]
pub struct Amount(pub u64);

impl From<u64> for Amount {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Amount {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn from_readable(value: f64, decimals: u8) -> Self {
        Self((value * 10u64.pow(decimals as u32) as f64) as u64)
    }

    pub fn to_readable(&self, decimals: u8) -> f64 {
        self.0 as f64 / 10u64.pow(decimals as u32) as f64
    }
}

impl Deref for Amount {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type Sender<T> = Arc<UnboundedSender<T>>;
pub type Receiver<T> = Arc<Mutex<UnboundedReceiver<T>>>;
