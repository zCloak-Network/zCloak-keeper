use frame_metadata::StorageHasher;
use keeper_primitives::{Deserialize, Serialize};
use jsonrpsee::{
    http_client::{HttpClient, HttpClientBuilder},
    types::{to_json_value, traits::Client, Error as RpcError},
};
pub use sp_core::{
    Bytes,
    H256 as Hash, storage::{StorageData, StorageKey},
};
use keeper_primitives::keeper::KeeperSetting;
use super::error::Error;
use prometheus_endpoint::Registry;
use crate::metrics::KiltMetrics;

pub const KILT_LOG_TARGET: &str = "KILT";
pub const ATTESTATION_PALLET_PREFIX: &'static str = "Attestation";
pub const ATTESTATION_STORAGE_PREFIX: &'static str = "Attestations";
pub const HASHER: StorageHasher = StorageHasher::Blake2_128Concat;
pub const KILT_MAX_RETRY_TIMES: usize = 5;

pub struct Service {
    pub metrics: Option<KiltMetrics>,
    pub registry: Option<Registry>,
    // client that handle connections
    pub client: KiltClient,
    pub keeper_setting: KeeperSetting,
}

impl Service {
    pub async fn new(url: &str) -> Self {
        let client = KiltClient::try_from_url(&url).await.expect("Fail to init kilt client");
        Self {
            metrics: None,
            registry: None,
            client,
            keeper_setting: Default::default()
        }
    }

    // inject prometheus metrics
    pub fn inject_metrics(mut self, metrics: KiltMetrics) -> Self {
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

#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize, Default)]
pub struct KiltConfig {
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct KiltClient {
    client: HttpClient,
    pub ip_address: String,
}

impl KiltClient {
    pub async fn try_from_url(url: &str) -> std::result::Result<Self, Error> {
        if url.starts_with("http://") || url.starts_with("https://") {
            let client = HttpClientBuilder::default().build(&url)?;
            Ok(KiltClient { client, ip_address: url.to_string() })
        } else {
            Err(Error::UrlFormatError(
                "Kilt client connection must start with http or https".to_owned(),
            ))
        }
    }

    // fetch storage
    pub async fn request_storage(
        &self,
        key: &StorageKey,
        hash: Option<Hash>,
    ) -> std::result::Result<Option<StorageData>, RpcError> {
        let params = vec![to_json_value(key)?, to_json_value(hash)?];
        let data = self.client.request("state_getStorage", Some(params.into())).await?;
        Ok(data)
    }
}




