use super::run::{
	query_ipfs,
	scan_moonbeam::{Bytes32, ProofEvent},
};
use hex;
use web3::types::Address;

#[inline]
fn ready_proof() -> ProofEvent {
	let program_hash_str = "0x8acf8f36dbd0407ced227c97f9f1bcf989c6affd32231ad56a36e9dfcd492610";
	let hex_program_hash = hex::decode(&program_hash_str[2..]).unwrap();
	let mut program_hash: [u8; 32] = Default::default();
	program_hash.copy_from_slice(&hex_program_hash);

	let ctype_str = "0x7f2ef721b292b9b7d678e9f82ab010e139600558df805bbc61a0041e60b61a18";
	let hex_ctype_hash = hex::decode(&ctype_str[2..]).unwrap();
	let mut ctype_hash: [u8; 32] = Default::default();
	ctype_hash.copy_from_slice(&hex_ctype_hash);

	let root_hash_str = "0x49a1c1c22ba0920ceb8c34184f5069cac9966116b85baa5160c0e70b092fe088";
	let hex_root_hash = hex::decode(&root_hash_str[2..]).unwrap();
	let mut root_hash: [u8; 32] = Default::default();
	root_hash.copy_from_slice(&hex_root_hash);

	ProofEvent {
		data_owner: Address::default(),
		kilt_address: Bytes32::default(),
		c_type: ctype_hash,
		program_hash,
		field_name: String::from("age"),
		proof_cid: String::from("QmWSnVGex9CXd2ZWY3nfrb6Liax6f1r5aRZbBoECox1hVD"),
		root_hash,
		expect_result: true,
	}
}

#[test]
fn test_starks_verify() {
	let public_outputs: Vec<u128> =
		vec![97873533375341971767141245126995634634, 267955640875355502483963951475246227592, 1];

	// fetched from http://ipfs.io/ipfs/QmWSnVGex9CXd2ZWY3nfrb6Liax6f1r5aRZbBoECox1hVD
	let proof_bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/proofs/.proof"));

	let proof_event = ready_proof();
	assert_eq!(proof_event.outputs(), public_outputs);

	let res = query_ipfs::verify(&proof_event, proof_bytes).unwrap();

	assert_eq!(res, true);
}
