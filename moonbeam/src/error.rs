
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Moonbeam connection Error: {0}")]
    ClientCreationError(String),

    #[error("Private Key Error, Error: {0}")]
    PrivateKeyError(#[from] secp256k1::Error),

    #[error("Web3 Client Error, err: {0}")]
    Web3Error(#[from] web3::Error),

    #[error("Web3 Contract Error, err: {0}")]
    Web3ContractError(#[from] web3::contract::Error),

    #[error("Ethereum Abi Error, err: {0}")]
    EthAbiError(#[from] web3::ethabi::Error),

    #[error("Invalid Ethereum Address: {0}")]
    InvalidEthereumAddress(String),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    // todo: make it generic
    #[error("Timeout error, err: {0}")]
    TimeOutError(#[from] tokio::time::error::Elapsed),
}