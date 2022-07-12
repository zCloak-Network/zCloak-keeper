use super::{
	Contract, Deserialize, Http, IpfsClient, IpfsConfig, KiltClient, KiltConfig, MoonbeamClient,
	MoonbeamConfig, Serialize,
};
use crate::monitor::MonitorConfig;
use secp256k1::SecretKey;
use std::{fs::File, path::PathBuf};

// todo: move
#[derive(Clone, Debug)]
pub struct ChannelFiles {
	pub event_to_ipfs: PathBuf,
	pub verify_to_attest: PathBuf,
	pub attest_to_submit: PathBuf,
	pub resubmit: PathBuf,
}

// todo move
#[derive(Clone, Debug)]
pub struct ConfigInstance {
	pub name: String,
	pub channel_files: ChannelFiles,
	pub moonbeam_client: MoonbeamClient,
	pub ipfs_client: IpfsClient,
	pub kilt_client: KiltClient,
	pub proof_contract: Contract<Http>,
	pub aggregator_contract: Contract<Http>,
	pub private_key: SecretKey,
	pub private_key_optional: Option<SecretKey>,
	#[cfg(feature = "monitor")]
	pub bot_url: String,
}

#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct Config {
	pub moonbeam: MoonbeamConfig,
	pub ipfs: IpfsConfig,
	pub kilt: KiltConfig,
	#[cfg(feature = "monitor")]
	pub monitor: MonitorConfig,
}

impl Config {
	pub fn load_from_json(config_path: &PathBuf) -> Result<Self> {
		let file = File::open(config_path)?;
		let res = serde_json::from_reader(file)?;
		Ok(res)
	}
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Open Config File Error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Json Parse to Config Error: {0}")]
	JsonParseError(#[from] serde_json::Error),
	#[error("Other Error: {0}")]
	OtherError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use crate::{monitor::MonitorConfig, Config};

	#[test]
	#[cfg(not(feature = "monitor"))]
	fn config_parse_should_work() {
		let path = PathBuf::from("./res/config-example.json");
		let config = Config::load_from_json(&path).unwrap();
		use crate::{IpfsConfig, KiltConfig, MoonbeamConfig};
		let expect = Config {
			moonbeam: MoonbeamConfig {
				url: "http://127.0.0.1:7545".to_string(),
				read_contract: "read_contract".to_string(),
				write_contract: "write_contract".to_string(),
				private_key: "private_key".to_string(),
			},
			ipfs: IpfsConfig { base_url: "https://ipfs.infura.io:5001".to_string() },
			kilt: KiltConfig { url: "kilt_url".to_string() },
		};

		assert_eq!(config, expect);
	}

	#[test]
	#[cfg(feature = "monitor")]
	fn config_load_in_feature_monitor_should_work() {
		let path = PathBuf::from("./res/config-example.json");
		let config = Config::load_from_json(&path).unwrap();
		assert_eq!(config.monitor, MonitorConfig { bot_url: "bot_url".to_owned() });
	}
}
