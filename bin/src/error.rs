use prometheus_endpoint::PrometheusError;
use keeper_primitives::traits::IntoStr;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Config load Error, err: {0}")]
	ConfigLoadError(#[from] super::config::Error),

	#[error("msg queue file I/O error,  err: {0}")]
	IoError(#[from] std::io::Error),

	#[error("Event Parse Error, err: {0}")]
	EventParseError(#[from] serde_json::Error),

	#[error(transparent)]
	MoonbeamError(#[from] moonbeam::Error),

	#[error(transparent)]
	IpfsError(#[from] ipfs::Error),

	#[error("StarksVM Verify Error, err: {0}")]
	StarksVMError(#[from] keeper_primitives::verify::Error),

	#[error(transparent)]
	KiltError(#[from] kilt::Error),

	#[error("Unexpect Error, err: {0}")]
	OtherError(String),

	#[error("Parse private Error, err: {0}")]
	PrivateKeyError(#[from] secp256k1::Error),

	#[error("Task error, err: {0}")]
	TaskJoinError(#[from] tokio::task::JoinError),

	// todo: unify all timeout error
	#[error("Timeout error, err: {0}")]
	TimeOutError(#[from] tokio::time::error::Elapsed),

	#[error("Prometheus Error, err: {0}")]
	PrometheusError(#[from] PrometheusError),
}
