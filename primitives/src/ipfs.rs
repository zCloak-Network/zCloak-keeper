use super::*;
use serde::{Deserialize, Serialize};
// ipfs max retry times
const IPFS_MAX_RETRY_TIMES: usize = 5;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct IpfsConfig {
    pub base_url: String,
}


pub struct IpfsClient {
    base_url: Url,
}


impl IpfsClient {
    pub fn new(url: String) -> Self {
        Self { base_url: Url::parse(&url).expect("host must can be convented into a valid url") }
    }

    pub async fn keep_fetch_proof(&self, proof_cid: &str) -> Result<Vec<u8>> {
        log::info!("[IPFS] start querying ipfs cid : {:?}", proof_cid);
        let mut url = build_request_url(&self.base_url, proof_cid)?;
        // just align with reqwest http request. if use other scheme should change this.
        url.set_scheme("https").map_err(|_| Error::SchemeError)?;

        log::debug!("file which on ipfs, url is {:?}", url);

        let mut times = 0;
        let body = loop {
            let maybe_response = reqwest::get(url.clone()).await;
            match maybe_response {
                Ok(r) => break r.text().await?,
                Err(e) => {
                    if e.is_timeout() && times < IPFS_MAX_RETRY_TIMES {
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

const IPFS_INFURA_IO: &'static str = "http://ipfs.infura.io:5001/";
const IPFS_INFUA_IO_PATH: &'static str = "api/v0/cat?arg=";
const IPFS_IO: &'static str = "https://ipfs.io/";
const IPFS_IO_PATH: &'static str = "ipfs";

fn build_request_url(base_url: &Url, cid: &str) -> Result<Url> {
    let url = match base_url.as_str() {
        IPFS_INFURA_IO => {
            // TODO improve this, now we add parameters just in a directly way. (arg=?)
            let path = IPFS_INFUA_IO_PATH.to_string() + cid;
            base_url.join(&path)?
        },
        IPFS_IO => {
            let base_url = Url::parse(IPFS_IO).unwrap();
            base_url.join(IPFS_IO_PATH)?.join(cid)?
        },
        _ => return Err(Error::InvalidIpfsHost),
    };
    Ok(url)
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

