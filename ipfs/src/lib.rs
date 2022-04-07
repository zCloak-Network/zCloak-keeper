use std::collections::BTreeMap;

use keeper_primitives::{
	ipfs::{IpfsClient, IPFS_LOG_TARGET},
	moonbeam::ProofEvent,
	verify::{verify_proof, Result, VERIFY_LOG_TARGET},
	Result as KeeperResult, VerifyResult, U64,
};

pub async fn query_and_verify(
	ipfs: &IpfsClient,
	input: BTreeMap<U64, Vec<ProofEvent>>,
) -> KeeperResult<Vec<VerifyResult>> {
	log::info!(target: IPFS_LOG_TARGET, "start querying ipfs");
	let mut ret = vec![];
	for (number, proofs) in input {
		for proof in proofs {
			let cid_context =
				ipfs.fetch_proof(proof.proof_cid()).await.map_err(|e| (number, e.into()))?;
			log::info!(
				target: IPFS_LOG_TARGET,
				"ipfs proof fetched and the content length is {}",
				cid_context.len()
			);
			// if verify meet error, do not throw it.
			let result = match verify(&proof, &cid_context) {
				Ok(r) => {
					if !r {
						// TODO set to database in future
						log::info!(
                            target: VERIFY_LOG_TARGET,
                            "verify zkStark from cid context failed|event_blocknumber:{:}|cid:{:}",
                            number, proof.proof_cid());
					}
					r
				},
				Err(e) => {
					log::error!(
						target: VERIFY_LOG_TARGET,
						"verify zkStark inner error|e:{:?}|event_blocknumber:{:}|cid:{:}",
						e,
						number,
						proof.proof_cid(),
					);
					false
				},
			};

			ret.push(VerifyResult::new_from_proof_event(proof, number, result));
		}
	}
	Ok(ret)
}

pub(crate) fn verify(p: &ProofEvent, context: &[u8]) -> Result<bool> {
	let inputs = p.public_inputs();
	let outputs = p.outputs();
	let program_hash = p.program_hash();
	let r = verify_proof(&program_hash, context, inputs.as_slice(), outputs.as_slice())?;
	log::info!(
		target: VERIFY_LOG_TARGET,
		"[STARKVM] the proof {:?} is verified as {}",
		p.proof_cid(),
		r
	);
	Ok(r)
}
