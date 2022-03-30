pub use web3::types::{Address, U64, BlockNumber, Log, FilterBuilder};
use web3::Web3;
pub use serde::{Serialize, Deserialize};
pub use sp_core::{
    storage::{StorageData, StorageKey},
    Bytes, H256 as Hash,
};
pub use codec::{Encode, Decode};
use web3::contract::{Error as ContractError, tokens::Detokenize};
use web3::ethabi::Token;
pub use moonbeam::{MoonbeamConfig, MoonbeamClient};
pub use kilt::{KiltConfig, KiltClient};
pub use ipfs::{IpfsConfig, IpfsClient};

pub mod moonbeam;
pub mod kilt;
pub mod ipfs;
pub mod error;
pub mod verify;


pub type Bytes32 = [u8; 32];
pub type Result<T> = std::result::Result<T, (U64, error::Error)>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub moonbeam: MoonbeamConfig,
    pub ipfs: IpfsConfig,
    pub kilt: KiltConfig,
}

// pub type ProofEventType = (Address, Bytes32, Bytes32, Bytes32, String, String, Bytes32, bool);

#[derive(Debug, Default)]
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
    fn from_tokens(mut tokens: Vec<Token>) -> std::result::Result<Self, web3::contract::Error> {
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
                expect_result: proof_event_enum.7
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


