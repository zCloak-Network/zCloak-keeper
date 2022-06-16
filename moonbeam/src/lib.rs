use log::error;
use keeper_primitives::{
	Events, Http, ProofEvent,
	VerifyResult, TIMEOUT_DURATION, Bytes32
};

use secp256k1::SecretKey;
pub use task::{task_scan, task_submit};

use web3::types::U64;
use keeper_primitives::{Serialize, Deserialize};
pub mod metrics;
mod task;
mod types;
mod utils;
mod funcs;
mod error;

pub use error::Error;
pub use types::{MOONBEAM_SCAN_LOG_TARGET, MOONBEAM_QUERY_LOG_TARGET, MOONBEAM_SUBMIT_LOG_TARGET};
pub use types::{Service as MoonbeamService, ServiceBuilder as MoonbeamServiceBuilder, MoonbeamConfig, MoonbeamClient};
type MoonbeamError = (Option<U64>, error::Error);

pub type MoonbeamResult<T> = std::result::Result<T, MoonbeamError>;