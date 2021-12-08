use component_ipfs::config::IpfsConfig;
use serde::{Deserialize, Serialize};
use server_traits::server::config::{Config, ServerConfig};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MoonbeamTaskConfig {
	pub moonbeam: MoonbeamConfig,
	pub contract: ContractConfig,
	pub ipfs: IpfsConfig,
	pub kilt: KiltConfig,
}

impl MoonbeamTaskConfig {
	pub fn store<S: AsRef<str>>(&self, sand_name: S) -> anyhow::Result<()> {
		let sand_name = sand_name.as_ref();
		Config::store_with_namespace(sand_name, self.moonbeam.clone(), "moonbeam")?;
		Config::store_with_namespace(sand_name, self.contract.clone(), "contract")?;
		Config::store_with_namespace(sand_name, self.ipfs.clone(), "ipfs")?;
		Config::store_with_namespace(sand_name, self.kilt.clone(), "kilt")?;
		Ok(())
	}

	pub fn template() -> Self {
		Self {
			moonbeam: MoonbeamConfig::template(),
			contract: ContractConfig::template(),
			ipfs: IpfsConfig::template(),
			kilt: KiltConfig::template(),
		}
	}
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MoonbeamConfig {
	pub url: String,
}

impl ServerConfig for MoonbeamConfig {
	fn marker() -> &'static str {
		"moonbeam"
	}

	fn template() -> Self {
		Self { url: "wss://127.0.0.1:9933".to_string() }
	}
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ContractConfig {
	pub address: String,
	pub topics: Vec<String>,
	pub password: String,
	pub uuid: String,
}

impl ServerConfig for ContractConfig {
	fn marker() -> &'static str {
		"contract"
	}

	fn template() -> Self {
		Self {
			address: "0x...".to_string(),
			topics: vec!["0x...".to_string()],
			password: "".to_string(),
			uuid: "".to_string(),
		}
	}
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KiltConfig {
	pub url: String,
	pub private_key: String,
}

impl ServerConfig for KiltConfig {
	fn marker() -> &'static str {
		"kilt"
	}

	fn template() -> Self {
		Self { url: "".to_string(), private_key: "0x...".to_string() }
	}
}
