use keeper_primitives::{
	ipfs::{IpfsClient, IPFS_LOG_TARGET},
	verify::{verify_proof, Result, VERIFY_LOG_TARGET},
	Events, ProofEvent, Result as KeeperResult, VerifyResult,
};
pub use task::task_verify;

mod task;

// empty return is set to none
pub async fn query_and_verify(
	ipfs: &IpfsClient,
	input: Events,
) -> KeeperResult<Option<Vec<VerifyResult>>> {
	log::info!(target: IPFS_LOG_TARGET, "start querying ipfs");
	let mut ret = vec![];
	for proof in input {
		let cid_context = ipfs
			.fetch_proof(proof.proof_cid())
			.await
			.map_err(|e| (proof.block_number(), e.into()))?;
		log::info!(
			target: IPFS_LOG_TARGET,
			"ipfs proof of data owner {:} in block {:?} fetched and the content length is {}",
			hex::encode(proof.data_owner()),
			proof.block_number(),
			cid_context.len()
		);
		// if verify meet error, do not throw it.
		let result = match verify(&proof, &cid_context) {
			Ok(r) => {
				log::info!(
					target: VERIFY_LOG_TARGET,
					"[STARKVM] the proof in block {:?}| cid {:?} | is verified as {:}",
					&proof.block_number(),
					proof.proof_cid(),
					r
				);
				if !r {
					// TODO set to database in future
				}
				r
			},
			Err(e) => {
				log::error!(
					target: VERIFY_LOG_TARGET,
					"verify zkStark inner error|e:{:?}|event_blocknumber:{:?}|cid:{:}",
					e,
					&proof.block_number(),
					proof.proof_cid(),
				);
				false
			},
		};

		ret.push(VerifyResult::new_from_proof_event(proof, result));
	}

	if ret.is_empty() {
		Ok(None)
	} else {
		Ok(Some(ret))
	}
}

pub(crate) fn verify(p: &ProofEvent, context: &[u8]) -> Result<bool> {
	let inputs = p.public_inputs();
	let outputs = p.outputs();
	let program_hash = p.program_hash();
	let r = verify_proof(&program_hash, context, inputs, &outputs)?;
	log::info!(
		target: VERIFY_LOG_TARGET,
		"[STARKVM] the proof {:?} is verified as {}",
		p.proof_cid(),
		r
	);
	Ok(r)
}
