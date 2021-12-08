use crate::utils::command::ToolsCommand;
use std::{
	fs,
	path::{Path, PathBuf},
};

use bip39::{Language, Mnemonic, MnemonicType, Seed};
use eth_keystore::{decrypt_key, encrypt_key, new};

pub async fn handle_tools(command: ToolsCommand) -> anyhow::Result<()> {
	match command {
		ToolsCommand::Keccak256 { inputs } => {
			let hash = keccak256(inputs.as_bytes());
			let hash = array_bytes::bytes2hex("", hash);
			println!("{:?} keccak256 hash value is:{:?}", inputs, hash);
		},
		ToolsCommand::Decrypt { path, password } => {
			let mnemonic = decrypt_keystore_file(password, path);
			println!("mnemonic {:?} ", mnemonic);
		},
		ToolsCommand::NewAccount { password } => {
			let (pk, uuid) = create_new_account(password);
			println!("pk is {:?}, uuid is {:?}", pk, uuid);
		},
	}
	Ok(())
}

/// Compute the Keccak-256 hash of input bytes.
pub fn keccak256(bytes: &[u8]) -> [u8; 32] {
	use tiny_keccak::{Hasher, Keccak};
	let mut output = [0u8; 32];
	let mut hasher = Keccak::v256();
	hasher.update(bytes);
	hasher.finalize(&mut output);
	output
}

pub fn decrypt_keystore_file(password: String, path: String) -> Result<Mnemonic, anyhow::Error> {
	println!("-----------");
	let key_path = Path::new(path.as_str());
	println!("path is {:?}", &key_path);
	let entropy = decrypt_key(&key_path, "ljy123456")?;
	println!("entropy is {:?}", &entropy);
	Mnemonic::from_entropy(&entropy, Language::English)
}

pub fn create_new_account(password: String) -> (Vec<u8>, String) {
	let dir = Path::new("./keys");
	let mut rng = rand::thread_rng();
	let (pk, uuid) = new(&dir, &mut rng, password).unwrap();
	return (pk, uuid)
}
