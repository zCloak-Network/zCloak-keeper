use starksVM as stark;

pub const VERIFY_LOG_TARGET: &str = "StarkVerify";

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

#[cfg(test)]
mod tests {

	use crate::ipfs::IpfsClient;

	#[tokio::test]
	async fn verify_should_work() {
		let program_hash = [
			85, 240, 19, 237, 111, 148, 74, 101, 238, 76, 236, 4, 253, 175, 28, 149, 160, 161, 81,
			162, 117, 180, 36, 64, 29, 56, 109, 193, 196, 236, 207, 254,
		];
		let proof_cid = "QmRFeY7ZeywFyXzT7pCR9ZGyZqhNs9y4ozhMGgSpvTAb4f";
		let public_inputs = [
			6383461,
			427020088179,
			8271117968073418672650679055481,
			30765223346328968342731846777,
			9459527121954502519414720132217,
		];
		let public_outputs = [
			138393280564113376992984738626752023914,
			281318377845317858458548900585840899268,
			1,
			0,
			0,
		];

		let ipfs_client =
			IpfsClient::new("https://ipfs.infura.io:5001").expect("ipfs client building fails.");
		let raw_proof = ipfs_client.fetch_proof(proof_cid).await.expect("wrong raw proof fetching");

		let res = super::verify_proof(&program_hash, &raw_proof, &public_inputs, &public_outputs);
		assert!(res.is_ok());
		assert!(res.unwrap());
	}
}
