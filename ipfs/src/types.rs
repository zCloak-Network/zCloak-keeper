
use keeper_primitives::keeper::KeeperSetting;
// todo: import serde
use keeper_primitives::{Serialize, Deserialize};
use prometheus_endpoint::Registry;
use reqwest::Client;
use std::time::Duration;
use super::metrics::IpfsMetrics;

pub const IPFS_LOG_TARGET: &str = "IPFS";

// ipfs max retry times
const IPFS_MAX_RETRY_TIMES: usize = 5;
const TIME_OUT: Duration = Duration::from_secs(5);
// TODO: move to config
const INFURA_USERNAME: &str = "26pucpYcATVSbrd7Cfvjwi2XcwT";
const INFURA_PASSWORD: &str = "9b3ca935d5c247e3fa9542f713498c91";
const IPFS_CAT_PATH: &str = "api/v0/cat";

use super::error::Error;

#[derive(Debug, Default)]
pub struct Service {
    pub metrics: Option<IpfsMetrics>,
    pub registry: Option<Registry>,
    // client that handle connections
    pub client: IpfsClient,
    pub keeper_setting: KeeperSetting,
}

impl Service {
    pub fn new(base_url: &str) -> Self {
        let client = IpfsClient::new(base_url).expect("Fail to init ipfs client");
        Self {client, ..Default::default()}
    }

    // inject prometheus metrics
    pub fn inject_metrics(mut self, metrics: IpfsMetrics) -> Self {
        self.metrics = Some(metrics);
        self
    }

    // inject prometheus registry
    pub fn inject_registry(mut self, registry: Registry) -> Self {
        self.registry = Some(registry);
        self
    }

    pub fn inject_keeper_setting(mut self, keeper: KeeperSetting) -> Self {
        self.keeper_setting = keeper;
        self
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct IpfsConfig {
    // e.g.  https://ipfs.infura.io:5001
    pub base_url: String,
}

// fixme: remove?
#[derive(Default)]
#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct IpfsClient {
    // e.g.  https://ipfs.infura.io:5001/api/v0/cat
    pub cat_url_prefix: String,
    pub ip_address: String,
}

impl IpfsClient {
    // e.g. https://ipfs.infura.io:5001/api/v0/cat
    pub fn new(config_base_url: &str) -> std::result::Result<Self, Error> {
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

    pub async fn fetch_proof(&self, cid: &str) -> std::result::Result<Vec<u8>, Error> {
        log::info!(target: IPFS_LOG_TARGET, "Start querying ipfs cid : {:?}", cid);

        let client = Client::builder().connect_timeout(TIME_OUT).build()?;
        keep_fetch(&self.cat_url_prefix, cid, client).await
    }
}

async fn keep_fetch(base_url: &str, cid: &str, client: Client) -> std::result::Result<Vec<u8>, Error> {
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

