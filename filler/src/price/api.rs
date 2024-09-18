use async_trait::async_trait;

use crate::error::Error;

#[async_trait]
pub trait PriceApi: Send + Sync {
    /// Get the prices of the given coin ids in USD
    async fn prices(&self, ids: &[&str]) -> Result<Vec<f64>, Error>;
}

