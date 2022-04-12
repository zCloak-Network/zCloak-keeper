use super::{Deserialize, IpfsConfig, KiltConfig, MoonbeamConfig, Serialize};
use std::{fs::File, path::PathBuf};

use super::{MoonbeamClient, IpfsClient, KiltClient, Contract, Http};
use secp256k1::SecretKey;

#[derive(Clone, Debug)]
pub struct ConfigInstance {
	pub moonbeam_client: MoonbeamClient,
	pub ipfs_client: IpfsClient,
	pub kilt_client: KiltClient,
	pub proof_contract: Contract<Http>,
	pub aggregator_contract: Contract<Http>,
	pub private_key: SecretKey,
}


#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct Config {
	pub moonbeam: MoonbeamConfig,
	pub ipfs: IpfsConfig,
	pub kilt: KiltConfig,
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
	use crate::{Config, IpfsConfig, KiltConfig, MoonbeamConfig};
	use std::path::PathBuf;
	#[test]
	fn config_parse_should_work() {
		let path = PathBuf::from("./res/config-example.json");
		let config = Config::load_from_json(&path).unwrap();
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
}
