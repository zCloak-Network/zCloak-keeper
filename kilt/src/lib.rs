use jsonrpsee::types::Error as RpcError;
use keeper_primitives::{
	Decode, Hash, U64
};
use std::time::Duration;
pub use task::task_attestation;
use tokio::time::{timeout_at, Instant};
pub use error::Error;
pub use types::Service as KiltService;
pub use types::{KiltClient, KiltConfig, KILT_LOG_TARGET};


mod task;
mod metrics;
mod error;
mod types;
mod funcs;
mod utils;

type KiltError = (Option<U64>, Error);

type Result<T> = std::result::Result<T, KiltError>;