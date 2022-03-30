#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Moonbeam Error, err: {0}")]
    MoonbeamError(#[from] crate::moonbeam::Error),

    #[error("Web3 Error, err: {0}")]
    Web3Error(#[from] web3::Error),

    #[error("Web3 Contract Error, err: {0}")]
    Web3ContractError(#[from] web3::contract::Error),

    #[error("Fetch IPFS Error, err: {0}")]
    IpfsError(#[from] crate::ipfs::Error),

    #[error("StarksVM Verify Error, err: {0}")]
    StarksVMError(#[from] crate::verify::Error),

    #[error("Fetch Kilt attestation Error, err: {0}")]
    KiltError(#[from] crate::kilt::Error),

    #[error("Unexpect Error, err: {0}")]
    OtherError(#[from] anyhow::Error),

    #[error("Parse private Error, err: {0}")]
    PrivateKeyError(#[from] secp256k1::Error),
}

