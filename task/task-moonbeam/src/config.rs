use primitives::utils::ipfs::config::IpfsConfig;
use serde::{Deserialize, Serialize};
use server_traits::server::config::{Config, ServerConfig};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MoonbeamTaskConfig {
	pub moonbeam: MoonbeamConfig,
	pub contract: ContractConfig,
	pub ipfs: IpfsConfig,

}

impl MoonbeamTaskConfig {
	pub fn store<S: AsRef<str>>(&self, sand_name: S) -> anyhow::Result<()> {
		let sand_name = sand_name.as_ref();
		Config::store_with_namespace(sand_name, self.moonbeam.clone(), "moonbeam")?;
		Config::store_with_namespace(sand_name, self.contract.clone(), "contract")?;
		Config::store_with_namespace(sand_name, self.ipfs.clone(), "ipfs")?;
		Ok(())
	}

	pub fn template() -> Self {
		Self { moonbeam: MoonbeamConfig::template(), contract: ContractConfig::template() ,ipfs: IpfsConfig::template()}
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
}


impl ServerConfig for ContractConfig {
    fn marker() ->&'static str {
        "contract"
    }

    fn template() -> Self {
        Self {
            address: "0x...".to_string(),
            topics: vec!["0x...".to_string()]
        }
    }
}
