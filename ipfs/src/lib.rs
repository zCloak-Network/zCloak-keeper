use keeper_primitives::{
	U64,
	verify::{verify_proof, VERIFY_LOG_TARGET},
	Events, ProofEvent, VerifyResult,
};
pub use task::task_verify;
pub use error::Error;
pub use types::Service as IpfsService;
pub use types::{IpfsClient, IpfsConfig};
pub use types::IPFS_LOG_TARGET;

mod task;
mod types;
mod error;
mod funcs;
mod metrics;


pub type IpfsError = (Option<U64>, Error);
pub type Result<T> = std::result::Result<T, IpfsError>;