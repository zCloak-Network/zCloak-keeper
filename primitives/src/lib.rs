pub use web3::types::{Address, U64, BlockNumber, Log, FilterBuilder};
use web3::Web3;
use reqwest::Url;
pub use serde::{Serialize, Deserialize};
pub use sp_core::{
    storage::{StorageData, StorageKey},
    Bytes, H256 as Hash,
};
pub use codec::{Encode, Decode};
pub use moonbeam::{MoonbeamConfig, MoonbeamClient, ProofEvent};
pub use kilt::{KiltConfig, KiltClient};
pub use ipfs::{IpfsConfig, IpfsClient};

pub mod moonbeam;
pub mod kilt;
pub mod ipfs;
pub mod error;
pub mod verify;


pub type Bytes32 = [u8; 32];


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub moonbeam: MoonbeamConfig,
    pub ipfs: IpfsConfig,
    pub kilt: KiltConfig,
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


