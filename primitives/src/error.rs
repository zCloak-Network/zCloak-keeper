#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Config load Error, err: {0}")]
	ConfigLoadError(#[from] crate::config::Error),

	#[error("msg queue file I/O error,  err: {0}")]
	IoError(#[from] std::io::Error),

	#[error("Event Parse Error, err: {0}")]
	EventParseError(#[from] serde_json::Error),

	#[error("Moonbeam Error, err: {0}")]
	MoonbeamError(#[from] crate::moonbeam::Error),

	#[error("Fetch IPFS Error, err: {0}")]
	IpfsError(#[from] crate::ipfs::Error),

	#[error("StarksVM Verify Error, err: {0}")]
	StarksVMError(#[from] crate::verify::Error),

	#[error("Fetch Kilt attestation Error, err: {0}")]
	KiltError(#[from] crate::kilt::Error),

	#[error("Unexpect Error, err: {0}")]
	OtherError(String),

	#[error("Parse private Error, err: {0}")]
	PrivateKeyError(#[from] secp256k1::Error),

	#[error("Task error, err: {0}")]
	TaskJoinError(#[from] tokio::task::JoinError),

	#[error(
		"Can not handle local receipt queue item, len: {0}, front item send_at block number:{1}."
	)]
	ExceedQueueLen(usize, u64),
}
