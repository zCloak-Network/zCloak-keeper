use std::collections::BTreeMap;

use secp256k1::SecretKey;
use web3::signing::{Key, SecretKeyRef};

use keeper_primitives::{
	moonbeam::{
		self, ProofEvent, MOONBEAM_LISTENED_EVENT, MOONBEAM_LOG_TARGET, MOONBEAM_SCAN_SPAN,
		MOONBEAM_TRANSACTION_CONFIRMATIONS,
	},
	Contract, Http, MoonbeamClient, Result as KeeperResult, VerifyResult, Web3Options, U64,
};

// scan moonbeam events
pub async fn scan_events(
	mut start: U64,
	best: U64,
	client: &MoonbeamClient,
	proof_contract: &Contract<Http>,
) -> KeeperResult<(BTreeMap<U64, Vec<ProofEvent>>, U64)> {
	// if start > best, reset `start` pointer to best
	if start > best {
		log::warn!(
			target: MOONBEAM_LOG_TARGET,
			"scan moonbeam start block is higher than current best! start_block={}, best_block:{}",
			start,
			best
		);
		start = best;
	}
	let span: U64 = MOONBEAM_SCAN_SPAN.into();
	let end = if start + span > best { best } else { start + span };

	log::info!(
		target: MOONBEAM_LOG_TARGET,
		"try to scan moonbeam log from block [{:}] - [{:}] | best:{:}",
		start,
		end,
		best
	);
	// parse event
	let r = moonbeam::utils::events::<_, ProofEvent>(
		client.eth(),
		proof_contract,
		MOONBEAM_LISTENED_EVENT,
		Some(start),
		Some(end),
	)
	.await;

	// if event parse error, return Err(start) and output error log
	let r = match r {
		Ok(events) => events,
		Err(err) => {
			log::error!(
				target: MOONBEAM_LOG_TARGET,
				"Moonbeam Scan Err: Event parse error. {:?}",
				err
			);
			return Err((start, err.into()))
		},
	};

	let hit = r.len();

	let mut result = BTreeMap::<U64, Vec<ProofEvent>>::default();
	for (proof_event, log) in r {
		let number = log.block_number.unwrap_or_else(|| {
			log::warn!(target: MOONBEAM_LOG_TARGET, "Moonbeam log blocknumber should not be None");
			Default::default()
		});

		result.entry(number).or_insert(vec![]).push(proof_event.clone().into());
		log::info!(
			target: MOONBEAM_LOG_TARGET,
			"[Moonbeam] event contains request hash: {:} | root hash: {:} has been recorded",
			hex::encode(proof_event.request_hash()),
			hex::encode(proof_event.root_hash())
		);
	}
	log::info!(
		"scan from [{:}] - [{:}] | hit:[{:}] | in blocks: {:?}",
		start,
		end,
		hit,
		result.keys().into_iter().map(|n| *n).collect::<Vec<U64>>()
	);
	Ok((result, end))
}

pub async fn submit_txs(
	contract: &Contract<Http>,
	worker: SecretKey,
	res: Vec<VerifyResult>,
) -> std::result::Result<(), keeper_primitives::moonbeam::Error> {
	log::info!(target: MOONBEAM_LOG_TARGET, "[Moonbeam] submitting the tx");
	let key_ref = SecretKeyRef::new(&worker);
	let worker_address = key_ref.address();
	for v in res {
		// TODO: read multiple times?
		let has_submitted: bool = contract
			.query(
				"hasSubmitted",
				(worker_address, v.request_hash),
				None,
				Web3Options::default(),
				None,
			)
			.await?;

		let is_finished: bool = contract
			.query(
				"isFinished",
				(v.data_owner, v.request_hash),
				None,
				Web3Options::default(),
				None,
			)
			.await?;

		log::info!(
			target: MOONBEAM_LOG_TARGET,
			"hasSubmitted result for request hash [{:?}] is {}, isFinished result is {:}",
			v.request_hash,
			has_submitted,
			is_finished
		);

		if !has_submitted && !is_finished {
			let r = contract
				.signed_call_with_confirmations(
					"submit",
					(v.data_owner, v.request_hash, v.c_type, v.root_hash, v.is_passed, v.attester),
					{
						let mut options = Web3Options::default();
						options.gas = Some(1000000.into());
						options
					},
					MOONBEAM_TRANSACTION_CONFIRMATIONS,
					&worker,
				)
				.await;

			match r {
				Ok(r) => {
					log::info!(
						target: MOONBEAM_LOG_TARGET,
						"Successfully submit verification|tx:{:}|data owner:{:}|root_hash:{:}|is_passed: {:}|attester: {:}",
						r.transaction_hash,
						v.data_owner,
						hex::encode(v.root_hash),
						v.is_passed,
						hex::encode(v.attester),
					)
				},
				Err(e) => {
					log::error!(
						target: MOONBEAM_LOG_TARGET,
						"Error submit verification |data owner:{:}|root_hash:{:}|request_hash: {:}, err: {:?}",
						v.data_owner,
						hex::encode(v.root_hash),
						hex::encode(v.request_hash),
						e
					)
				},
			}
		}
	}

	Ok(())
}
