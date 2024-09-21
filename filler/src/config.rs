use fuels::types::{AssetId, ContractId};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use crate::error::Error;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetConfig {
    /// Asset name (BTC, ETH, etc.)
    pub name: String,

    /// Asset decimals
    pub decimals: u8,

    /// Price API id
    pub price_id: String,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Indexer websocket host url
    pub indexer_ws_host: String,

    /// Coingecko API host url
    pub coingecko_host: String,

    /// Strategy interval in milliseconds
    pub interval: u64,

    /// Number of traders to run
    pub traders_num: usize,

    /// Maximum number of calls in multicall transaction
    pub multicall_size: usize,

    /// Spark Market contract IDs
    pub markets: Vec<ContractId>,

    /// All available assets
    #[serde_as(as = "HashMap<_, _>")]
    pub assets: HashMap<AssetId, AssetConfig>,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;

        Ok(config)
    }
}
