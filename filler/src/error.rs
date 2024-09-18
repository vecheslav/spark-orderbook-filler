use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO: {0}")]
    IO(#[from] std::io::Error),

    #[error("ParseInt: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("ParseChrono: {0}")]
    ParseChrono(#[from] chrono::ParseError),

    #[error("Json: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Websocket: {0}")]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Fuel error: {0}")]
    Fuel(#[from] fuels::types::errors::Error),

    #[error("Price API: {0}")]
    PriceApi(String),
}
