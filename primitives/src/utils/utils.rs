use starksVM as stark;

pub fn verifier_proof(
	program_hash: &[u8; 32],
	body: Vec<u8>,
	public_inputs: &[u128],
	outputs: &[u128],
) -> anyhow::Result<bool> {
	let hexed_proof = hex::decode(&body[0..body.len()])?;
	let stark_proof = bincode::deserialize::<stark::StarkProof>(&hexed_proof)?;

	let maybe_result = stark::verify(program_hash, public_inputs, outputs, &stark_proof);

	match maybe_result {
		Ok(res) => Ok(res),
		Err(e) => Err(anyhow::Error::msg(e)),
	}
}
