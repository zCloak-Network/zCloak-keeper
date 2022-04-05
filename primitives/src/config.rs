use super::{Deserialize, IpfsConfig, KiltConfig, MoonbeamConfig, Serialize};
use std::{fs::File, path::PathBuf};

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
		let path = PathBuf::from("./res/config.json");
		let config = Config::load_from_json(&path).unwrap();
		let expect = Config {
			moonbeam: MoonbeamConfig {
				url: "http://127.0.0.1".to_string(),
				read_contract: "mock_read_contract".to_string(),
				write_contract: "mock_write_contract".to_string(),
				private_key: "0xxxx".to_string(),
			},
			ipfs: IpfsConfig { base_url: "ipfs_mock_url".to_string() },
			kilt: KiltConfig { url: "mock_kilt_url".to_string() },
		};

		assert_eq!(config, expect);
	}
}
