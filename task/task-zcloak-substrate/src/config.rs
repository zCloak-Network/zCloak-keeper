use primitives::utils::ipfs::config::IpfsConfig;
use serde::{Deserialize, Serialize};
use server_traits::server::config::{Config, ServerConfig};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ZcloakTaskConfig {
	pub zcloak: ZcloakNodeConfig,
	pub ipfs: IpfsConfig,
}

impl ZcloakTaskConfig {
	pub fn store<S: AsRef<str>>(&self, sand_name: S) -> anyhow::Result<()> {
		let sand_name = sand_name.as_ref();
		Config::store_with_namespace(sand_name, self.zcloak.clone(), "zcloak")?;
		Config::store_with_namespace(sand_name, self.ipfs.clone(), "ipfs")?;
		Ok(())
	}

	pub fn template() -> Self {
		Self { zcloak: ZcloakNodeConfig::template(), ipfs: IpfsConfig::template() }
	}
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ZcloakNodeConfig {
	pub url: String,
	pub private_key: String,
}

impl ServerConfig for ZcloakNodeConfig {
	fn marker() -> &'static str {
		"zcloak-node"
	}

	fn template() -> Self {
		Self { url: "wss://test1.zcloak.network".to_string(), private_key: "0x...".to_string() }
	}
}
