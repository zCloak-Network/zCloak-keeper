use starksVM as stark;

pub fn verify_proof(
	program_hash: &[u8; 32],
	body: &[u8],
	public_inputs: &[u128],
	outputs: &[u128],
) -> Result<bool> {
	let hexed_proof = hex::decode(body)?;
	let stark_proof = bincode::deserialize::<stark::StarkProof>(&hexed_proof)?;

	let maybe_result = stark::verify(program_hash, public_inputs, outputs, &stark_proof);

	match maybe_result {
		Ok(res) => Ok(res),
		Err(e) => Err(Error::VerifyError(e)),
	}
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Hex Decode Error: err{0}")]
	HexError(#[from] hex::FromHexError),
	#[error("Parse hex into StarkProof Error: err{0}")]
	StarkProofDeserializeError(#[from] bincode::Error),
	#[error("StarksVM Verify Error: err{0}")]
	VerifyError(String),
}
pub type Result<T> = std::result::Result<T, Error>;
