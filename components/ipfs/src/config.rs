use serde::{Deserialize, Serialize};
use server_traits::server::config::ServerConfig;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct IpfsConfig {
	pub host: String,
}

impl ServerConfig for IpfsConfig {
	fn marker() -> &'static str {
		"config-ipfs"
	}

	fn template() -> Self {
		Self { host: "ipfs.infura.io:5001".to_string() }
	}
}
