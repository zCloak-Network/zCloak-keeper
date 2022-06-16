use super::{Deserialize, Serialize, *};
use codec::{Decode, Encode};
use frame_metadata::StorageHasher;
use jsonrpsee::{
	http_client::{HttpClient, HttpClientBuilder},
	types::{to_json_value, traits::Client, Error as RpcError},
};
use sp_runtime::AccountId32 as AccountId;

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
