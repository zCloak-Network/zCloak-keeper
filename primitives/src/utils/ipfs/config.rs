use serde::{Deserialize, Serialize};
use server_traits::server::config::ServerConfig;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct IpfsConfig {
    pub url_index: String,
}

impl ServerConfig for IpfsConfig {
    fn marker() -> &'static str {
        "config-ipfs"
    }

    fn template() -> Self {
        Self {
            url_index: "https://ipfs.infura.io:5001/api/v0/cat?arg=".to_string(),
        }
    }
}


