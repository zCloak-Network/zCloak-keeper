use codec::{Decode, Encode};
use frame_metadata::StorageHasher;
use jsonrpsee::{
	http_client::{HttpClient, HttpClientBuilder},
	types::{Error as RpcError, to_json_value, traits::Client},
};
use sp_runtime::AccountId32 as AccountId;
use super::{Serialize, Deserialize};
use super::*;

pub const KILT_LOG_TARGET: &str = "KILT";
const ATTESTATION_PALLET_PREFIX: &'static str = "Attestation";
const ATTESTATION_STORAGE_PREFIX: &'static str = "Attestations";
const HASHER: StorageHasher = StorageHasher::Blake2_128Concat;
pub const KILT_MAX_RETRY_TIMES: usize = 5;

//fixme: make generic
pub type Balance = u128;

#[derive(Default, Clone, Debug, Encode, Decode, PartialEq, Serialize, Deserialize)]
pub struct Deposit<Account, Balance> {
	pub owner: Account,
	pub amount: Balance,
}

#[derive(Default, Clone, Debug, Encode, Decode, PartialEq, Serialize, Deserialize)]
pub struct AttestationDetails<Hash: Encode + Clone, Account, Balance> {
	pub ctype_hash: Hash,
	pub attester: Account,
	pub delegation_id: Option<Hash>,
	pub revoked: bool,
	pub deposit: Deposit<Account, Balance>,
}

pub type Attestation = AttestationDetails<Hash, AccountId, Balance>;

#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct KiltConfig {
	pub url: String,
}

#[derive(Clone, Debug)]
pub struct KiltClient {
	client: HttpClient,
}

impl KiltClient {
	pub async fn try_from_url(url: &str) -> Result<Self> {
		if url.starts_with("http://") || url.starts_with("https://") {
			let client = HttpClientBuilder::default().build(&url)?;
			Ok(KiltClient { client })
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
	bytes.extend(key_hash(&key, &HASHER));
	StorageKey(bytes)
}

#[allow(unused)]
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
	#[error("Wrong url initializing client connection, err: {0}")]
	UrlFormatError(String),
	#[error("Kilt RPC Client Error, err: {0}")]
	KiltClientError(#[from] RpcError),
	/// Serde serialization error
	#[error("Serde json error: {0}")]
	Serialization(#[from] serde_json::error::Error),
	#[error("Error decoding storage value: {0}")]
	StorageValueDecode(#[from] codec::Error),
}

type Result<T> = std::result::Result<T, Error>;
