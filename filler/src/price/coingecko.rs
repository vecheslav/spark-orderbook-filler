use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::error::Error;

use super::PriceApi;

pub struct CoingeckoApi {
    pub host: String,
    pub api_key: String,
    pub client: Client,
}

#[async_trait]
impl PriceApi for CoingeckoApi {
    /// Get the prices of the given coin ids in USD
    async fn prices(&self, ids: &[&str]) -> Result<Vec<f64>, Error> {
        let price_map = self.simple_price(ids, &["usd"]).await?;
        Ok(ids
            .iter()
            .map(|&id| price_map[id].usd.unwrap())
            .collect::<Vec<_>>())
    }
}

impl CoingeckoApi {
    pub fn new(host: String, api_key: String) -> Self {
        Self {
            host,
            api_key,
            client: Client::new(),
        }
    }

    /// Generic method to make a GET request to the API
    async fn get<R: DeserializeOwned>(&self, endpoint: &str) -> Result<R, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(
            // TODO: Replace demo name
            "x-cg-demo-api-key",
            HeaderValue::from_str(&self.api_key).map_err(|e| Error::PriceApi(e.to_string()))?,
        );

        let res = self
            .client
            .get(&format!("{}{}", self.host, endpoint))
            .headers(headers)
            .send()
            .await?
            .json()
            .await?;

        Ok(res)
    }

    /// Get the simple price of the given coin ids in the given vs currencies
    pub async fn simple_price<Id: AsRef<str>, Curr: AsRef<str>>(
        &self,
        ids: &[Id],
        vs_currencies: &[Curr],
    ) -> Result<HashMap<String, Price>, Error> {
        let ids = ids.iter().map(AsRef::as_ref).collect::<Vec<_>>();
        let vs_currencies = vs_currencies.iter().map(AsRef::as_ref).collect::<Vec<_>>();

        let req = format!(
            "/simple/price?ids={}&vs_currencies={}&precision=full",
            ids.join("%2C"),
            vs_currencies.join("%2C")
        );

        self.get(&req).await
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Price {
    pub btc: Option<f64>,
    pub eth: Option<f64>,
    pub usd: Option<f64>,
}
