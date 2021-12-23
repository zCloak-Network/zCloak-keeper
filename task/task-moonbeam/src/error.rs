#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Web3 Error, err: {0}")]
	Web3Error(#[from] web3::Error),

	#[error("Web3 Contract Error, err: {0}")]
	Web3ContractError(#[from] web3::contract::Error),

	#[error("Ethereum Abi Error, err: {0}")]
	EthAbiError(#[from] web3::ethabi::Error),

	#[error("Invalid Ethereum Address: {0}")]
	InvalidEthereumAddress(String),

	#[error("Fetch IPFS Error, err: {0}")]
	IpfsError(#[from] component_ipfs::Error),

	#[error("Fetch Kilt attestation Error, err: {0}")]
	KiltError(#[from] support_kilt_node::Error),

	#[error("Unexpect Error, err: {0}")]
	OtherError(#[from] anyhow::Error),

	#[error("Parse private Error, err: {0}")]
	PrivateKeyError(#[from] secp256k1::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
