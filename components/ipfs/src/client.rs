use crate::{Error, Result};
use reqwest::Url;
use std::str;

pub struct IpfsClient {
	host: Url,
}

const MAX_RETRY_TIMES: usize = 5;

impl IpfsClient {
	pub fn new(url: String) -> Self {
		Self { host: Url::parse(&url).expect("host must can be convented into a valid url") }
	}

	pub async fn keep_fetch_proof(&self, proof_cid: &str) -> Result<Vec<u8>> {
		let mut url = build_request_url(&self.host, proof_cid)?;
		// just align with reqwest http request. if use other scheme should change this.
		url.set_scheme("https").map_err(|_| Error::SchemeError)?;

		log::debug!("file which on ipfs, url is {:?}", url);

		let mut times = 0;
		let body = loop {
			let maybe_response = reqwest::get(url.clone()).await;
			match maybe_response {
				Ok(r) => {
					break r.text().await?
				},
				Err(e) => {
					if e.is_timeout() && times < MAX_RETRY_TIMES {
						log::warn!("ipfs client fetch data timeout! retry: {:} ...", times + 1);
						times += 1;
						continue
					}
					log::error!("ipfs client fetch data error. reason: {:?}", e);
					Err(e)?
				},
			}
		};

		let body = body.as_bytes().to_owned();
		Ok(body)
	}
}

const IPFS_INFURA_IO: &'static str = "ipfs.infura.io:5001";
const IPFS_INFUA_IO_PATH: &'static str = "api/v0/cat?arg=";
const IPFS_IO: &'static str = "ipfs.io";
const IPFS_IO_PATH: &'static str = "ipfs";

fn build_request_url(host: &Url, cid: &str) -> Result<Url> {
	let url = match host.as_str() {
		IPFS_INFURA_IO => {
			// TODO improve this, now we add parameters just in a directly way. (arg=?)
			let path = IPFS_INFUA_IO_PATH.to_string() + cid;
			host.join(&path)?
		},
		IPFS_IO => host.join(IPFS_IO_PATH)?.join(cid)?,
		_ => return Err(Error::InvalidIpfsHost),
	};
	Ok(url)
}
