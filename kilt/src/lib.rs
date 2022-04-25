use jsonrpsee::types::Error as RpcError;

use keeper_primitives::{
	Decode,
	Hash, kilt::{
		Attestation, Error, get_attestation_storage_key, KILT_LOG_TARGET, KILT_MAX_RETRY_TIMES,
		KiltClient,
	}, Result, VerifyResult,
};
pub use task::task_attestation;

mod task;

pub async fn filter(client: &KiltClient, result: Vec<VerifyResult>) -> Result<Vec<VerifyResult>> {
	let mut v = vec![];
	for i in result {
		// query attestation details from kilt
		let maybe_attest = query_attestation(client, i.root_hash.into())
			.await
			.map_err(|e| (i.number, e.into()))?;

		let mut v_update = i.clone();

		// submit to moonbeam if and only if
		// the attestation neither empty or revoked
		if maybe_attest.is_some() &&
			Ok(()) == v_update.update_from_attestation(maybe_attest.unwrap())
		{
			log::info!(
				target: KILT_LOG_TARGET,
				"roothash: {:} | in block #{:?} has been attested",
				hex::encode(i.root_hash),
				i.number
			);
			v.push(v_update)
		} else {
			log::warn!(
                target: KILT_LOG_TARGET,
                "attestaion is not valid for this root_hash|root_hash:{:}|data owner:{:}|number:{:?}",
				hex::encode(i.root_hash), hex::encode(i.data_owner.0),
				i.number
			);
			// TODO: notice the server that user's credential is not valid on kilt
		}
	}
	Ok(v)
}

/// query attestation info from kilt network
/// TODO: handle kilt error??
pub async fn query_attestation(
	client: &KiltClient,
	root_hash: Hash,
) -> std::result::Result<Option<Attestation>, Error> {
	let storage_key = get_attestation_storage_key::<Hash>(root_hash);
	let mut times = 0;
	let maybe_attestation_details = loop {
		// connect to kilt and query attestation storage
		match client.request_storage(&storage_key, None).await {
			Ok(details) => break details,
			Err(e) => {
				match e {
					RpcError::RequestTimeout | RpcError::Transport(_) =>
						if times < KILT_MAX_RETRY_TIMES {
							times += 1;
							log::warn!(
								target: KILT_LOG_TARGET,
								"query kilt storage timeout, retry {:}/{:}",
								times,
								KILT_MAX_RETRY_TIMES
							);
							continue
						},

					_ => {},
				}
				return Err(e)?
			},
		}
	};

	// decode fetched storage data
	let maybe_attestation: Option<Attestation> = match maybe_attestation_details {
		Some(data) => Some(Decode::decode(&mut data.0.as_slice())?),
		None => None,
	};

	log::info!(
		target: KILT_LOG_TARGET,
		"Kilt query result of roothash: [{:}] is {:?}",
		hex::encode(root_hash),
		maybe_attestation
	);

	Ok(maybe_attestation)
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use keeper_primitives::{kilt::Attestation, KiltClient};

	use crate::{Hash, query_attestation};

	#[inline]
	fn attest_exp() -> Attestation {
		let json_attest = r#"
		{
  			"ctype_hash": "0x7f2ef721b292b9b7d678e9f82ab010e139600558df805bbc61a0041e60b61a18",
  			"attester": "4pf1fzQjRnNcLxKDn6vE1R3nBEu1MUbW4Wz63uKFQUD7WHR2",
  			"delegationId": "null",
  			"revoked": false,
  			"deposit": {
    			"owner": "4pf1fzQjRnNcLxKDn6vE1R3nBEu1MUbW4Wz63uKFQUD7WHR2",
    			"amount": 120900000000000
  			}
		}"#;

		let attest_transformed =
			serde_json::from_str(json_attest).expect("wrong parse to attestation");
		attest_transformed
	}

	#[tokio::test]
	async fn valid_root_hash_should_work() {
		let attest_exp = attest_exp();

		let kilt_client = KiltClient::try_from_url("http://45.77.247.174:40011/")
			.await
			.expect("wrong url for kilt client");
		let right_root_hash =
			Hash::from_str("af6e8c774b0f7409743f7e28e29fd3196d0eee72c66e57c550302abea4336933")
				.expect("root hash from string error");
		let maybe_right_attest = query_attestation(&kilt_client, right_root_hash).await;
		assert_eq!(maybe_right_attest.unwrap().unwrap(), attest_exp);

		let empty_root_hash =
			Hash::from_str("7b6e8c774b0f7409743f7e28e29fd3196d0eee72c66e57c550302abea4336966")
				.expect("root hash from string error");
		let maybe_empty_attest = query_attestation(&kilt_client, empty_root_hash).await;
		assert_eq!(maybe_empty_attest.unwrap(), None);
	}

	#[tokio::test]
	async fn invalid_root_hash_should_work() {
		let kilt_client = KiltClient::try_from_url("http://45.77.247.174:40011/")
			.await
			.expect("wrong url for kilt client");

		let empty_root_hash =
			Hash::from_str("7b6e8c774b0f7409743f7e28e29fd3196d0eee72c66e57c550302abea4336966")
				.expect("root hash from string error");
		let maybe_empty_attest = query_attestation(&kilt_client, empty_root_hash).await;
		assert_eq!(maybe_empty_attest.unwrap(), None);
	}
}
