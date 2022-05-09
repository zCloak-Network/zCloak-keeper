use super::{Deserialize, Serialize};
use reqwest::Client;
use std::time::Duration;

pub const IPFS_LOG_TARGET: &str = "IPFS";

// ipfs max retry times
const IPFS_MAX_RETRY_TIMES: usize = 5;
const TIME_OUT: Duration = Duration::from_secs(5);
// TODO:
const INFURA_USERNAME: &str = "26pucpYcATVSbrd7Cfvjwi2XcwT";
const INFURA_PASSWORD: &str = "9b3ca935d5c247e3fa9542f713498c91";
const IPFS_CAT_PATH: &str = "api/v0/cat";

#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct IpfsConfig {
	// e.g.  https://ipfs.infura.io:5001
	pub base_url: String,
}

// fixme: remove?
#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct IpfsClient {
	// e.g.  https://ipfs.infura.io:5001/api/v0/cat
	pub cat_url_prefix: String,
	pub ip_address: String,
}

impl IpfsClient {
	// e.g. https://ipfs.infura.io:5001/api/v0/cat
	pub fn new(config_base_url: &str) -> Result<Self> {
		if config_base_url.starts_with("https") {
			let cat_url = if !config_base_url.ends_with("/") {
				config_base_url.to_owned() + "/"
			} else {
				config_base_url.to_owned()
			};
			return Ok(IpfsClient {
				cat_url_prefix: cat_url + IPFS_CAT_PATH,
				ip_address: String::from(config_base_url),
			})
		} else {
			return Err(Error::InvalidIpfsHost)
		}
	}

	pub async fn fetch_proof(&self, cid: &str) -> Result<Vec<u8>> {
		log::info!(target: IPFS_LOG_TARGET, "Start querying ipfs cid : {:?}", cid);

		let client = Client::builder().connect_timeout(TIME_OUT).build()?;
		keep_fetch(&self.cat_url_prefix, cid, client).await
	}
}

async fn keep_fetch(base_url: &str, cid: &str, client: Client) -> Result<Vec<u8>> {
	// TODO: make it config?
	let params = [("arg", cid)];
	let mut body = String::new();

	for i in 0..IPFS_MAX_RETRY_TIMES {
		let maybe_response = client
			.post(base_url)
			.query(&params)
			.basic_auth(INFURA_USERNAME, Some(INFURA_PASSWORD))
			.send()
			.await;
		match maybe_response {
			Ok(r) => {
				body = r.text().await?;
				break
			},
			Err(e) => {
				if e.is_timeout() && i < (IPFS_MAX_RETRY_TIMES - 1) {
					log::warn!("ipfs client fetch data timeout! retry: {:} ...", i + 1);
					continue
				}
				log::error!("ipfs client fetch data error. reason: {:?}", e);
				Err(e)?
			},
		}
	}
	Ok(body.into_bytes())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Just allow specified ipfs host")]
	InvalidIpfsHost,
	#[error("Request IPFS error, reason: {0}")]
	HttpError(#[from] reqwest::Error),
	#[error("Assembly Url error, reason: {0}")]
	UrlError(#[from] url::ParseError),
	#[error("Set Scheme Error")]
	SchemeError,
}

pub type Result<T> = std::result::Result<T, Error>;
