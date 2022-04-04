use std::collections::BTreeMap;

pub use codec::{Decode, Encode};
pub use serde::{Deserialize, Serialize};
pub use sp_core::{
    Bytes,
    H256 as Hash, storage::{StorageData, StorageKey},
};
pub use web3::{contract::{Contract, Options as Web3Options}, transports::Http};
use web3::contract::{Error as ContractError, tokens::Detokenize};
use web3::ethabi::Token;
pub use web3::types::{Address, BlockNumber, FilterBuilder, Log, U64};
use web3::types::Res;
use web3::Web3;

pub use config::Config;
pub use error::Error;
pub use ipfs::{IpfsClient, IpfsConfig};
pub use kilt::{KiltClient, KiltConfig};
pub use moonbeam::{MoonbeamClient, MoonbeamConfig};
pub use traits::JsonParse;

pub mod moonbeam;
pub mod kilt;
pub mod ipfs;
pub mod error;
pub mod verify;
pub mod config;
mod traits;


pub type Bytes32 = [u8; 32];
pub type Result<T> = std::result::Result<T, (U64, error::Error)>;


#[derive(PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct ProofEvent {
    pub(crate) data_owner: Address,
    pub(crate) kilt_address: Bytes32,
    pub(crate) c_type: Bytes32,
    pub(crate) program_hash: Bytes32,
    pub(crate) field_name: String,
    pub(crate) proof_cid: String,
    pub(crate) root_hash: Bytes32,
    pub(crate) expect_result: bool,
}

impl Detokenize for ProofEvent {
    fn from_tokens(tokens: Vec<Token>) -> std::result::Result<Self, web3::contract::Error> {
        if tokens.len() != 8 {
            return Err(ContractError::InvalidOutputType(format!(
                "Expected {} elements, got a list of {}: {:?}",
                8,
                tokens.len(),
                tokens
            )));
        }
        // TODO: make it config
        pub type ProofEventEnum = (Address, Bytes32, Bytes32, Bytes32, String, String, Bytes32, bool);
        let proof_event_enum = ProofEventEnum::from_tokens(tokens)?;
        Ok(
            ProofEvent {
                data_owner: proof_event_enum.0,
                kilt_address: proof_event_enum.1,
                c_type: proof_event_enum.2,
                program_hash: proof_event_enum.3,
                field_name: proof_event_enum.4,
                proof_cid: proof_event_enum.5,
                root_hash: proof_event_enum.6,
                expect_result: proof_event_enum.7,
            }
        )
    }
}

impl ProofEvent {
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
        if self.expect_result {
            outputs.push(1)
        } else {
            outputs.push(0)
        }

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

#[derive(PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct VerifyResult {
    pub number: U64,
    pub data_owner: Address,
    pub root_hash: Bytes32,
    pub c_type: Bytes32,
    pub program_hash: Bytes32,
    pub is_passed: bool,
}

impl VerifyResult {
    pub fn new_from_proof_event(p: ProofEvent, number: U64, passed: bool) -> Self {
        VerifyResult {
            number,
            data_owner: p.data_owner,
            root_hash: p.root_hash,
            c_type: p.c_type,
            program_hash: p.program_hash,
            is_passed: passed,
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::{EventResult, ProofEvent, traits::JsonParse, VerifyResult};

    #[test]
    fn event_result_parse_should_work() {
        let json_str = r#"{"0x1":[{"data_owner":"0x0000000000000000000000000000000000000000","kilt_address":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"c_type":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"program_hash":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"field_name":"","proof_cid":"","root_hash":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"expect_result":false}]}"#;
        let mut test_event = EventResult::new();
        test_event.entry(1.into()).or_insert(vec![ProofEvent::default()]);

        let event_str = test_event.into_json_str().unwrap();
        assert_eq!(std::str::from_utf8(&event_str).unwrap(), json_str);

        let event_res = EventResult::from_json_str(json_str.as_bytes());
        let event_res_value = event_res.get_key_value(&1u32.into()).unwrap().1;
        let test_event_value = event_res.get_key_value(&1u32.into()).unwrap().1;
        assert_eq!(*event_res_value, *test_event_value);
    }

    #[test]
    fn verify_result_parse_should_work() {
        let exp_verify_result_str = r#"{"number":"0x0","data_owner":"0x0000000000000000000000000000000000000000","root_hash":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"c_type":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"program_hash":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"is_passed":false}"#;
        let exp_verify_result_bytes = exp_verify_result_str.as_bytes();
        let v_res = VerifyResult::default();
        let v_res_bytes = serde_json::to_vec(&v_res).unwrap();
        assert_eq!(std::str::from_utf8(&v_res_bytes).unwrap(), exp_verify_result_str);

        let v_res_str_decoded: VerifyResult = serde_json::from_str(&exp_verify_result_str).unwrap();
        let v_res_bytes_decoded: VerifyResult = serde_json::from_slice(&v_res_bytes).unwrap();
        assert_eq!(v_res_bytes_decoded, v_res);
        assert_eq!(v_res_str_decoded, v_res);

    }
}