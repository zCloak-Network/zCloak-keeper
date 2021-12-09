use lifeline::Message;
use postage::broadcast;
use serde::{Deserialize, Serialize};
use web3::types::{Address, H256};

use crate::bus::MoonbeamTaskBus;
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddProof {
	pub user: Address,
	pub c_type: H256,
	pub program_hash: [u8; 32],
	// field_name
	pub public_input: Vec<u128>,
	// roothash and result
	pub public_output: Vec<u128>,
	pub proof_cid: Vec<u8>,
	pub expected_result: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Attestation {
	pub user: Address,
	pub c_type: H256,
	pub program_hash: H256,
	pub root_hash: H256,
	pub is_passed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MoonbeamTaskMessage {
	ListenMoonbeam,
	IpfsProof(AddProof),
	KiltAttestation(Attestation),
	SubmitVerification(Attestation),
}

impl Message<MoonbeamTaskBus> for MoonbeamTaskMessage {
	type Channel = broadcast::Sender<Self>;
}
