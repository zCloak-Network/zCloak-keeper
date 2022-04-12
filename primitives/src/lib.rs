pub use codec::{Decode, Encode};
pub use serde::{Deserialize, Serialize};
pub use sp_core::{
	storage::{StorageData, StorageKey},
	Bytes, H256 as Hash,
};
use std::{collections::BTreeMap, default::Default};
use web3::{
	contract::{tokens::Detokenize, Error as ContractError},
	ethabi::Token,
	types::Res,
	Web3,
};
pub use web3::{
	contract::{Contract, Options as Web3Options},
	transports::Http,
	types::{Address, BlockNumber, FilterBuilder, Log, U64},
};
pub use futures_timer::Delay;
use crate::kilt::Attestation;
pub use config::{Config, ConfigInstance};
pub use error::Error;
pub use ipfs::{IpfsClient, IpfsConfig};
pub use kilt::{KiltClient, KiltConfig};
pub use moonbeam::{MoonbeamClient, MoonbeamConfig};
pub use traits::JsonParse;
pub use yaque::{Sender as MqSender, Receiver as MqReceiver};

pub mod config;
pub mod error;
pub mod ipfs;
pub mod kilt;
// #[cfg(feature = "monitor")]
pub mod monitor;
pub mod moonbeam;
mod traits;
pub mod verify;

// todo: move
pub const CHANNEL_LOG_TARGET: &str = "Channel";
pub const MESSAGE_PARSE_LOG_TARGET: &str = "Message Parse";

pub type Bytes32 = [u8; 32];
pub type Result<T> = std::result::Result<T, (U64, error::Error)>;

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProofEvent {
	pub(crate) data_owner: Address,
	pub(crate) attester: Bytes32,
	pub(crate) c_type: Bytes32,
	pub(crate) program_hash: Bytes32,
	pub(crate) field_name: String,
	pub(crate) proof_cid: String,
	pub(crate) request_hash: Bytes32,
	pub(crate) root_hash: Bytes32,
	pub(crate) expect_result: Vec<u128>,
}

// # of elements in AddProof event
const EVENT_LEN: usize = 9;
// TODO: make it config
pub type ProofEventEnum =
	(Address, Bytes32, Bytes32, Bytes32, String, String, Bytes32, Bytes32, Vec<u128>);

impl Detokenize for ProofEvent {
	fn from_tokens(tokens: Vec<Token>) -> std::result::Result<Self, web3::contract::Error> {
		if tokens.len() != EVENT_LEN {
			return Err(ContractError::InvalidOutputType(format!(
				"Expected {} elements, got a list of {}: {:?}",
				8,
				tokens.len(),
				tokens
			)))
		}

		let proof_event_enum = ProofEventEnum::from_tokens(tokens)?;
		Ok(ProofEvent {
			data_owner: proof_event_enum.0,
			attester: proof_event_enum.1,
			c_type: proof_event_enum.2,
			program_hash: proof_event_enum.3,
			field_name: proof_event_enum.4,
			proof_cid: proof_event_enum.5,
			request_hash: proof_event_enum.6,
			root_hash: proof_event_enum.7,
			expect_result: proof_event_enum.8,
		})
	}
}

impl ProofEvent {
	pub fn request_hash(&self) -> Bytes32 {
		self.request_hash
	}

	// todo error handle?
	// calc the output from `ProofEvent`,
	// [exp_result_0, exp_result_1] -> [u8; 32]
	pub fn root_hash(&self) -> Bytes32 {
		self.root_hash
	}

	pub fn proof_cid(&self) -> &str {
		self.proof_cid.as_str()
	}
	// transform field name into u128 as public inputs
	pub fn public_inputs(&self) -> Vec<u128> {
		let hex_str = hex::encode(&self.field_name);
		let r = u128::from_str_radix(&hex_str, 16)
			.expect("filed_name from event must be fit into u128 range");
		// TODO in future, other params can be part of the inputs
		vec![r]
	}

	// calc the output from `ProofEvent`,
	// [rootHash_part1, rootHash_part2, verify_result]
	pub fn outputs(&self) -> Vec<u128> {
		let mut outputs = vec![];
		let mut mid: [u8; 16] = Default::default();
		mid.copy_from_slice(&self.root_hash[0..16]);
		outputs.push(u128::from_be_bytes(mid));
		mid.copy_from_slice(&self.root_hash[16..]);
		outputs.push(u128::from_be_bytes(mid));
		outputs.append(&mut self.expect_result.clone());

		outputs
	}

	pub fn program_hash(&self) -> Bytes32 {
		self.program_hash
	}
}

pub type EventResult = BTreeMap<U64, Vec<ProofEvent>>;

impl traits::JsonParse for EventResult {
	fn into_bytes(self) -> std::result::Result<Vec<u8>, error::Error> {
		serde_json::to_vec(&self).map_err(|e| e.into())
	}

	fn try_from_bytes(json: &[u8]) -> std::result::Result<Self, error::Error> {
		serde_json::from_slice(json).map_err(|e| e.into())
	}
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
	pub number: U64,
	pub data_owner: Address,
	pub root_hash: Bytes32,
	pub c_type: Bytes32,
	pub program_hash: Bytes32,
	pub request_hash: Bytes32,
	pub attester: Bytes32,
	pub is_passed: bool,
	// exp_result in ProofEvent
	pub calc_output: Vec<u128>,
}

impl VerifyResult {
	pub fn new_from_proof_event(p: ProofEvent, number: U64, passed: bool) -> Self {
		VerifyResult {
			number,
			data_owner: p.data_owner,
			root_hash: p.root_hash,
			c_type: p.c_type,
			program_hash: p.program_hash,
			request_hash: p.request_hash,
			attester: p.attester,
			is_passed: passed,
			calc_output: p.expect_result,
		}
	}

	// if the credential is revoked by attester, do nothing and return error
	pub fn update_from_attestation(&mut self, attest: Attestation) -> std::result::Result<(), ()> {
		if !attest.revoked {
			self.c_type = Bytes32::from(attest.ctype_hash);
			self.attester = Bytes32::from(attest.attester);
			Ok(())
		} else {
			Err(())
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::{traits::JsonParse, Bytes32, EventResult, ProofEvent, VerifyResult};
	use std::str::FromStr;
	use web3::types::Address;

	#[test]
	fn proof_event_parse_should_work() {
		let json_str = r#"{"data_owner":"0x127221418abcd357022d29f62449d98d9610dfab", "attester":[76,253,46,114,43,55,11,16,21,52,58,39,201,120,152,21,216,3,253,177,132,10,170,4,6,162,107,229,90,149,255,1],"c_type":[127,46,247,33,178,146,185,183,214,120,233,248,42,176,16,225,57,96,5,88,223,128,91,188,97,160,4,30,96,182,26,24],"program_hash":[138,207,143,54,219,208,64,124,237,34,124,151,249,241,188,249,137,198,175,253,50,35,26,213,106,54,233,223,205,73,38,16],"field_name":"age","proof_cid":"QmUn4UfXdv7uJXerqy1PMfnXxYuM3xfpUC8pFZaVyJoN7H","request_hash":[94,173,49,247,138,238,243,148,66,124,21,189,107,13,78,210,69,212,74,170,249,110,90,37,128,46,16,119,10,76,17,117],"root_hash": [175, 110, 140, 119, 75, 15, 116, 9, 116, 63, 126, 40, 226, 159, 211, 25, 109, 14, 238,	114, 198, 110, 87, 197, 80, 48, 42, 190, 164, 51, 105, 51], "expect_result":[1]}"#;
		let mut test_event: ProofEvent =
			serde_json::from_str(json_str).expect("wrong proof event json");

		let root_hash = test_event.root_hash();
		assert_eq!(
			hex::encode(root_hash),
			"af6e8c774b0f7409743f7e28e29fd3196d0eee72c66e57c550302abea4336933"
		);
	}

	#[test]
	fn verify_result_parse_should_work() {
		let exp_verify_result_str = r#"{"number":"0x0","data_owner":"0x0000000000000000000000000000000000000000","root_hash":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"c_type":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"program_hash":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"request_hash":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"attester":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"is_passed":false}"#;
		let exp_verify_result_bytes = exp_verify_result_str.as_bytes();
		let v_res = VerifyResult::default();
		let v_res_bytes = serde_json::to_vec(&v_res).unwrap();
		assert_eq!(std::str::from_utf8(&v_res_bytes).unwrap(), exp_verify_result_str);

		let v_res_str_decoded: VerifyResult = serde_json::from_str(&exp_verify_result_str).unwrap();
		let v_res_bytes_decoded: VerifyResult = serde_json::from_slice(&v_res_bytes).unwrap();
		assert_eq!(v_res_bytes_decoded, v_res);
		assert_eq!(v_res_str_decoded, v_res);
	}

	#[test]
	fn event_result_parse_should_work() {
		let json_str = r#"{"0x21":[{"data_owner":"0x127221418abcd357022d29f62449d98d9610dfab","attester":[76,253,46,114,43,55,11,16,21,52,58,39,201,120,152,21,216,3,253,177,132,10,170,4,6,162,107,229,90,149,255,1],"c_type":[127,46,247,33,178,146,185,183,214,120,233,248,42,176,16,225,57,96,5,88,223,128,91,188,97,160,4,30,96,182,26,24],"program_hash":[138,207,143,54,219,208,64,124,237,34,124,151,249,241,188,249,137,198,175,253,50,35,26,213,106,54,233,223,205,73,38,16],"field_name":"age","proof_cid":"QmUn4UfXdv7uJXerqy1PMfnXxYuM3xfpUC8pFZaVyJoN7H","request_hash":[94,173,49,247,138,238,243,148,66,124,21,189,107,13,78,210,69,212,74,170,249,110,90,37,128,46,16,119,10,76,17,117],"root_hash":[175,110,140,119,75,15,116,9,116,63,126,40,226,159,211,25,109,14,238,114,198,110,87,197,80,48,42,190,164,51,105,51],"expect_result":[1]}]}"#;
		let mut test_event = EventResult::new();

		test_event.entry(33.into()).or_insert(vec![]).push(ProofEvent {
			data_owner: Address::from_str("0x127221418abcd357022d29f62449d98d9610dfab")
				.expect("wrong address"),
			attester: [
				76, 253, 46, 114, 43, 55, 11, 16, 21, 52, 58, 39, 201, 120, 152, 21, 216, 3, 253,
				177, 132, 10, 170, 4, 6, 162, 107, 229, 90, 149, 255, 1,
			],
			c_type: [
				127, 46, 247, 33, 178, 146, 185, 183, 214, 120, 233, 248, 42, 176, 16, 225, 57, 96,
				5, 88, 223, 128, 91, 188, 97, 160, 4, 30, 96, 182, 26, 24,
			],
			program_hash: [
				138, 207, 143, 54, 219, 208, 64, 124, 237, 34, 124, 151, 249, 241, 188, 249, 137,
				198, 175, 253, 50, 35, 26, 213, 106, 54, 233, 223, 205, 73, 38, 16,
			],
			field_name: "age".to_string(),
			proof_cid: "QmUn4UfXdv7uJXerqy1PMfnXxYuM3xfpUC8pFZaVyJoN7H".to_string(),
			request_hash: [
				94, 173, 49, 247, 138, 238, 243, 148, 66, 124, 21, 189, 107, 13, 78, 210, 69, 212,
				74, 170, 249, 110, 90, 37, 128, 46, 16, 119, 10, 76, 17, 117,
			],
			root_hash: [
				175, 110, 140, 119, 75, 15, 116, 9, 116, 63, 126, 40, 226, 159, 211, 25, 109, 14,
				238, 114, 198, 110, 87, 197, 80, 48, 42, 190, 164, 51, 105, 51,
			],
			expect_result: vec![1],
		});

		let event_str = test_event.into_bytes().unwrap();
		assert_eq!(std::str::from_utf8(&event_str).unwrap(), json_str);

		let event_res = EventResult::try_from_bytes(json_str.as_bytes()).unwrap();
		let event_res_value = event_res.get_key_value(&33.into()).unwrap().1;
		let test_event_value = event_res.get_key_value(&33.into()).unwrap().1;
		assert_eq!(*event_res_value, *test_event_value);
	}

	#[test]
	fn bytes32_segament_parse_should_correct() {
		// 6b696c744163636f756e74000000000000000000000000000000000000000000
		let kilt_address_slice = [
			107, 105, 108, 116, 65, 99, 99, 111, 117, 110, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0,
		];
		// 4cfd2e722b370b1015343a27c9789815d803fdb1840aaa0406a26be55a95ff01
		let attester_slice = [
			76, 253, 46, 114, 43, 55, 11, 16, 21, 52, 58, 39, 201, 120, 152, 21, 216, 3, 253, 177,
			132, 10, 170, 4, 6, 162, 107, 229, 90, 149, 255, 1,
		];
		// 7f2ef721b292b9b7d678e9f82ab010e139600558df805bbc61a0041e60b61a18
		let c_type_slice = [
			127, 46, 247, 33, 178, 146, 185, 183, 214, 120, 233, 248, 42, 176, 16, 225, 57, 96, 5,
			88, 223, 128, 91, 188, 97, 160, 4, 30, 96, 182, 26, 24,
		];
		// 8acf8f36dbd0407ced227c97f9f1bcf989c6affd32231ad56a36e9dfcd492610
		let program_hash_slice = [
			138, 207, 143, 54, 219, 208, 64, 124, 237, 34, 124, 151, 249, 241, 188, 249, 137, 198,
			175, 253, 50, 35, 26, 213, 106, 54, 233, 223, 205, 73, 38, 16,
		];
		let request_hash_slice = [
			94, 173, 49, 247, 138, 238, 243, 148, 66, 124, 21, 189, 107, 13, 78, 210, 69, 212, 74,
			170, 249, 110, 90, 37, 128, 46, 16, 119, 10, 76, 17, 117,
		];
		// af6e8c774b0f7409743f7e28e29fd3196d0eee72c66e57c550302abea4336933
		let root_hash_slice = [
			175, 110, 140, 119, 75, 15, 116, 9, 116, 63, 126, 40, 226, 159, 211, 25, 109, 14, 238,
			114, 198, 110, 87, 197, 80, 48, 42, 190, 164, 51, 105, 51,
		];

		// variables stored in zcloak-contracts/srcipts/variables.js
		assert_eq!(
			hex::encode(attester_slice),
			"4cfd2e722b370b1015343a27c9789815d803fdb1840aaa0406a26be55a95ff01"
		);
		assert_eq!(
			hex::encode(c_type_slice),
			"7f2ef721b292b9b7d678e9f82ab010e139600558df805bbc61a0041e60b61a18"
		);
		assert_eq!(
			hex::encode(program_hash_slice),
			"8acf8f36dbd0407ced227c97f9f1bcf989c6affd32231ad56a36e9dfcd492610"
		);
		assert_eq!(
			hex::encode(root_hash_slice),
			"af6e8c774b0f7409743f7e28e29fd3196d0eee72c66e57c550302abea4336933"
		);
	}
}
