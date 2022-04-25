use std::path::PathBuf;

use structopt::StructOpt;

use keeper_primitives::{config::Error as ConfigError, ChannelFiles};

const EVENT_TO_IPFS_CHANNEL: &str = "event2ipfs";
const VERIFY_TO_ATTEST_CHANNEL: &str = "verify2attest";
const ATTEST_TO_SUBMIT_CHANNEL: &str = "attest2submit";

#[derive(Debug, StructOpt)]
#[structopt(name = "zcloak Keeper", about = "zCloak keeper node start config")]
pub enum Opt {
	///start zCloak Server
	Start {
		#[structopt(flatten)]
		options: StartOptions,
	},
}

#[derive(Debug, Clone, StructOpt)]
pub struct StartOptions {
	///The zCloak keeper node config file path
	#[structopt(long, parse(from_os_str))]
	pub config: Option<PathBuf>,

	///The zCloak keeper node msg queue cache directory
	#[structopt(long, parse(from_os_str))]
	pub cache_dir: Option<PathBuf>,

	/// The starting block number of scanning node events.
	#[structopt(short, long)]
	pub start_number: Option<u64>,
}

impl StartOptions {
	pub(crate) fn channel_files(&self) -> std::result::Result<ChannelFiles, ConfigError> {
		match &self.cache_dir {
			Some(dir) => {
				let event_to_ipfs = dir.join(EVENT_TO_IPFS_CHANNEL);
				let verify_to_attest = dir.join(VERIFY_TO_ATTEST_CHANNEL);
				let attest_to_submit = dir.join(ATTEST_TO_SUBMIT_CHANNEL);
				Ok(ChannelFiles { event_to_ipfs, verify_to_attest, attest_to_submit })
			},
			None => Err(ConfigError::OtherError("Fail to create channel files.".to_owned())),
		}
	}
}
