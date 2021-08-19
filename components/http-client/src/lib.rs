use serde::{Deserialize, Serialize};
use server_traits::{
	error::ServerResult,
	server::{
		component::ServerComponent,
		config::{Config, ServerConfig},
		task::ServerSand,
	},
};
use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub struct HttpClientComponent {
	config: HttpClientConfig,
}

impl HttpClientComponent {
	pub fn new(config: HttpClientConfig) -> Self {
		Self { config }
	}
}

#[async_trait::async_trait]
impl ServerComponent<HttpClientConfig, reqwest::Client> for HttpClientComponent {
	fn restore_with_namespace<T: ServerSand>(namespace: String) -> ServerResult<Self> {
		let config: HttpClientConfig = Config::restore_with_namespace(T::NAME, namespace)?;
		Ok(Self::new(config))
	}

	async fn component(&self) -> anyhow::Result<reqwest::Client> {
		let client = reqwest::Client::builder()
			.timeout(Duration::from_secs(self.config.timeout))
			.build()?;
		Ok(client)
	}

	fn config(&self) -> &HttpClientConfig {
		&self.config
	}
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HttpClientConfig {
	pub timeout: u64,
}

impl ServerConfig for HttpClientConfig {
	fn marker() -> &'static str {
		"component-http-client"
	}

	fn template() -> Self {
		Self { timeout: 3000 }
	}
}
