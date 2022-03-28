use super::*;
use codec::{Decode, Encode};
use serde::{Serialize, Deserialize};
use frame_metadata::StorageHasher;
use jsonrpsee::{
    http_client::{HttpClient, HttpClientBuilder},
    types::{to_json_value, traits::Client, DeserializeOwned, Error as RpcError, JsonValue},
    ws_client::{WsClient, WsClientBuilder},
};

use sp_runtime::AccountId32 as AccountId;

//fixme: make generic
pub type Balance = u128;

const ATTESTATION_PALLET_PREFIX: &'static str = "Attestation";
const ATTESTATION_STORAGE_PREFIX: &'static str = "Attestations";
const Hasher: StorageHasher = StorageHasher::Blake2_128Concat;
pub const KILT_MAX_RETRY_TIMES: usize = 5;

#[derive(Default, Clone, Debug, Encode, Decode, PartialEq)]
pub struct Deposit<Account, Balance> {
    pub owner: Account,
    pub amount: Balance,
}


#[derive(Default, Clone, Debug, Encode, Decode, PartialEq)]
pub struct AttestationDetails<Hash: Encode + Clone, Account, Balance> {
    pub ctype_hash: Hash,
    pub attester: Account,
    pub delegation_id: Option<Hash>,
    pub revoked: bool,
    pub deposit: Deposit<Account, Balance>,
}

pub type Attestation = AttestationDetails<Hash, AccountId, Balance>;



#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KiltConfig {
    pub url: String,
}

pub enum KiltClient {
    WebSocket(WsClient),
    Http(HttpClient),
}

impl KiltClient {
    pub async fn try_from_url(url: &str) -> std::result::Result<Self, RpcError> {
        if url.starts_with("ws://") || url.starts_with("wss://") {
            let client =
                WsClientBuilder::default().max_notifs_per_subscription(4096).build(url).await?;
            Ok(KiltClient::WebSocket(client))
        } else {
            let client = HttpClientBuilder::default().build(&url)?;
            Ok(KiltClient::Http(client))
        }
    }

    // fetch storage
    pub async fn storage(
        &self,
        key: &StorageKey,
        hash: Option<Hash>,
    ) -> std::result::Result<Option<StorageData>, RpcError> {
        let params = vec![to_json_value(key)?, to_json_value(hash)?];
        let data = match self {
            Self::WebSocket(inner) =>
                inner.request("state_getStorage", Some(params.into())).await?,

            Self::Http(inner) => inner.request("state_getStorage", Some(params.into())).await?,
        };
        Ok(data)
    }
}


/// get the storage key of attestations
pub fn get_attestation_storage_key<Key: Encode>(key: Key) -> StorageKey {
    get_storage_map_key(key, ATTESTATION_PALLET_PREFIX, ATTESTATION_STORAGE_PREFIX)
}

fn get_storage_map_key<Key: Encode>(
    key: Key,
    pallet_prefix: &str,
    storage_prefix: &str,
) -> StorageKey {
    let mut bytes = sp_core::twox_128(pallet_prefix.as_bytes()).to_vec();
    bytes.extend(&sp_core::twox_128(storage_prefix.as_bytes())[..]);
    bytes.extend(key_hash(&key, &Hasher));
    StorageKey(bytes)
}

fn get_storage_value_key(pallet_prefix: &str, storage_prefix: &str) -> StorageKey {
    let mut bytes = sp_core::twox_128(pallet_prefix.as_bytes()).to_vec();
    bytes.extend(&sp_core::twox_128(storage_prefix.as_bytes())[..]);
    StorageKey(bytes)
}

fn key_hash<K: Encode>(key: &K, hasher: &StorageHasher) -> Vec<u8> {
    let encoded_key = key.encode();
    match hasher {
        StorageHasher::Identity => encoded_key.to_vec(),
        StorageHasher::Blake2_128 => sp_core::blake2_128(&encoded_key).to_vec(),
        StorageHasher::Blake2_128Concat => {
            // copied from substrate Blake2_128Concat::hash since StorageHasher is not public
            let x: &[u8] = encoded_key.as_slice();
            sp_core::blake2_128(x).iter().chain(x.iter()).cloned().collect::<Vec<_>>()
        },
        StorageHasher::Blake2_256 => sp_core::blake2_256(&encoded_key).to_vec(),
        StorageHasher::Twox128 => sp_core::twox_128(&encoded_key).to_vec(),
        StorageHasher::Twox256 => sp_core::twox_256(&encoded_key).to_vec(),
        StorageHasher::Twox64Concat =>
            sp_core::twox_64(&encoded_key).iter().chain(&encoded_key).cloned().collect(),
    }
}


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Kilt RPC Client Error, err: {0}")]
    KiltClientError(#[from] jsonrpsee::types::Error),
    /// Serde serialization error
    #[error("Serde json error: {0}")]
    Serialization(#[from] serde_json::error::Error),
    #[error("Error decoding storage value: {0}")]
    StorageValueDecode(#[from] codec::Error),
}