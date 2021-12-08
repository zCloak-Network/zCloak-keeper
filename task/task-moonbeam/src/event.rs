use crate::error::Error;
use core::str::FromStr;
use std::convert::{TryFrom, TryInto};
use subxt::sp_core::bytes::to_hex;
use web3::{
	ethabi::LogParam,
	types::{Address, H256, U128},
};

#[derive(Debug, Default)]
pub struct CreateTaskEvent {
	pub sender: Address,
	pub program_hash: [u8; 32],
	pub public_inputs: Vec<u128>,
	pub outputs: Vec<u128>,
	pub proof_id: Vec<u8>,
	pub program: Vec<u8>,
	pub c_type: [u8; 32],
	pub kilt_address: [u8; 32],
	pub root_hash: H256,
}

impl CreateTaskEvent {
	pub fn parse_log(params: Vec<LogParam>) -> anyhow::Result<CreateTaskEvent, anyhow::Error> {
		let mut create_param = CreateTaskEvent::default();
		for param in params {
			match param.name.as_str() {
				"dataOwner" => {
					create_param.sender = param.value.into_address().unwrap();
				},
				"kiltAddress" => {
					let bytes = param.value.into_fixed_bytes().unwrap();
					let buf = pop(&bytes[..]);
					match buf {
						Ok(buf) => {
							create_param.kilt_address = buf;
						},
						Err(e) => return Err(e),
					}
				},
				"cType" => {
					let bytes = param.value.into_fixed_bytes().unwrap();
					let buf = pop(&bytes[..]);
					match buf {
						Ok(buf) => {
							create_param.c_type = buf;
						},
						Err(e) => return Err(e),
					}
				},
				"programHash" => {
					let bytes = param.value.into_fixed_bytes().unwrap();
					let buf = pop(&bytes[..]);
					match buf {
						Ok(buf) => {
							create_param.program_hash = buf;
						},
						Err(e) => return Err(e),
					}
					create_param.program = bytes.clone();
				},
				"fieldName" => {
					let field_value = param.value.into_string().unwrap();
					let hex_value = hex::encode(field_value.clone());
					let value = u128::from_str_radix(&hex_value, 16).unwrap();
					create_param.public_inputs.push(value);
				},
				"expectResult" => {
					let value = param.value.into_bool().unwrap();
					if value {
						create_param.outputs.push(1);
					} else {
						create_param.outputs.push(0);
					}
				},
				"proofCid" => {
					let proof_cid = param.value.into_string().unwrap();
					create_param.proof_id = proof_cid.as_bytes().to_vec();
				},
				"rootHash" => {
					let bytes = param.value.into_fixed_bytes().unwrap();
					create_param.root_hash = H256::from_slice(&bytes[..]);
				},
				_ => {},
			}
		}
		log::info!("create param is {:?}", create_param);
		return Ok(create_param)
	}
}

fn pop(barry: &[u8]) -> anyhow::Result<[u8; 32], anyhow::Error> {
	let r = barry.try_into();
	match r {
		Ok(r) => return Ok(r),
		Err(e) =>
			return Err(Error::ParseLog(String::from("param - programHash is wrong !")).into()),
	}
}
