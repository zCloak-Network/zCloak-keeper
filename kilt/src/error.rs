use jsonrpsee::types::Error as RpcError;
use keeper_primitives::traits::IntoStr;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Wrong url initializing client connection, err: {0}")]
    UrlFormatError(String),

    #[error("Kilt RPC Client Error, err: {0}")]
    KiltClientError(#[from] RpcError),

    /// Serde serialization error
    #[error("Serde json error: {0}")]
    Serialization(#[from] serde_json::error::Error),

    #[error("Error decoding storage value: {0}")]
    StorageValueDecode(#[from] codec::Error),

    #[error("Timeout error, err: {0}")]
    TimeOutError(#[from] tokio::time::error::Elapsed),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
