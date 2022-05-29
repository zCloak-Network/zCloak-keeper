use keeper_primitives::{
	moonbeam::{
		self, utils::query_submit_and_finish_result, Events, ProofEvent, IS_FINISHED,
		MOONBEAM_LISTENED_EVENT, MOONBEAM_SCAN_LOG_TARGET, MOONBEAM_SCAN_SPAN,
		MOONBEAM_SUBMIT_LOG_TARGET, MOONBEAM_TRANSACTION_CONFIRMATIONS, SUBMIT_STATUS_QUERY,
		SUBMIT_TX_MAX_RETRY_TIMES, SUBMIT_VERIFICATION,
	},
	Address, Contract, Http, MoonbeamClient, Result as KeeperResult, VerifyResult, Web3Options,
	TIMEOUT_DURATION, U64,
};
use secp256k1::SecretKey;
pub use task::{task_scan, task_submit};
use tokio::time::{timeout_at, Instant};
pub mod metrics;
mod task;

// scan moonbeam events
pub async fn scan_events(
	mut start: U64,
	best: U64,
	client: &MoonbeamClient,
	proof_contract: &Contract<Http>,
) -> KeeperResult<(Option<Events>, U64)> {
	// if start > best, reset `start` pointer to best
	if start > best {
		log::warn!(
			target: MOONBEAM_SCAN_LOG_TARGET,
			"scan moonbeam start block is higher than current best! start_block={}, best_block:{}",
			start,
			best
		);
		start = best;
	}
	let span: U64 = MOONBEAM_SCAN_SPAN.into();
	let end = if start + span > best { best } else { start + span };

	log::info!(
		target: MOONBEAM_SCAN_LOG_TARGET,
		"try to scan moonbeam log from block [{:}] - [{:}] | best:{:}",
		start,
		end,
		best
	);
	// parse event
	let r = timeout_at(
		Instant::now() + TIMEOUT_DURATION,
		moonbeam::utils::events::<_, ProofEvent>(
			client.eth(),
			proof_contract,
			MOONBEAM_LISTENED_EVENT,
			Some(start),
			Some(end),
		),
	)
	.await
	.map_err(|e| (Some(start), e.into()))?;

	// if event parse error, return Err(start) and output error log
	let res = match r {
		Ok(events) => events,
		Err(err) => {
			log::error!(
				target: MOONBEAM_SCAN_LOG_TARGET,
				"Moonbeam Scan Err: Event parse error. {:?}",
				err
			);
			return Err((Some(start), err.into()))
		},
	};

	let hit = res.len();

	if hit != 0 {
		let mut result = vec![];
		for (mut proof_event, log) in res {
			let number = log.block_number;
			// warn
			if number.is_none() {
				log::warn!(
					target: MOONBEAM_SCAN_LOG_TARGET,
					"Moonbeam log block number should not be None"
				);
			}

			log::info!(
				"scan from [{:}] - [{:}] | hit:[{:}] | in blocks: {:?}",
				start,
				end,
				hit,
				number
			);

			// complete proof event
			proof_event.set_block_number(number);

			result.push(proof_event.clone());
			log::info!(
				target: MOONBEAM_SCAN_LOG_TARGET,
				"event in block {:?} contains data owner: {:} | request hash: {:} | root hash: {:} | program hash is {:} | calc output {:?} have been recorded",
				number,
				hex::encode(proof_event.data_owner()),
				hex::encode(proof_event.request_hash()),
				hex::encode(proof_event.root_hash()),
				hex::encode(proof_event.program_hash()),
				proof_event.raw_outputs()
			);
		}

		Ok((Some(result), end))
	} else {
		Ok((None, end))
	}
}

pub async fn submit_txs(
	contract: &Contract<Http>,
	keeper_pri: SecretKey,
	keeper_address: Address,
	res: Vec<VerifyResult>,
) -> std::result::Result<(), (Option<U64>, keeper_primitives::moonbeam::Error)> {
	for v in res {
		let query_submit_and_finish_results = query_submit_and_finish_result(
			contract,
			SUBMIT_STATUS_QUERY,
			(keeper_address, v.data_owner, v.request_hash),
			IS_FINISHED,
			(v.data_owner, v.request_hash),
			v.request_hash,
			SUBMIT_TX_MAX_RETRY_TIMES,
		)
		.await;

		match query_submit_and_finish_results {
			Ok((has_submitted, is_finished)) => {
				log::info!(
					target: MOONBEAM_SUBMIT_LOG_TARGET,
					"hasSubmitted result for request hash [{:?}] is {:?}, isFinished result is {:?}",
					hex::encode(v.request_hash),
					has_submitted,
					is_finished,
				);

				if !has_submitted && !is_finished {
					log::info!(
						target: MOONBEAM_SUBMIT_LOG_TARGET,
						"Start submitting: tx which contains user address: {:} |request_hash: {:}| root hash : {:} | isPassed: {}",
						v.data_owner,
						hex::encode(v.request_hash),
						hex::encode(v.root_hash),
						v.is_passed
					);

					let r = timeout_at(
						Instant::now() + TIMEOUT_DURATION,
						contract.signed_call_with_confirmations(
							SUBMIT_VERIFICATION,
							(
								v.data_owner,
								v.request_hash,
								v.c_type,
								v.root_hash,
								v.is_passed,
								v.attester,
								v.calc_output,
							),
							{
								// todo: auto adjust options here
								let mut options = Web3Options::default();
								options.gas = Some(1000000.into());
								options
							},
							MOONBEAM_TRANSACTION_CONFIRMATIONS,
							&keeper_pri,
						),
					)
					.await
					.map_err(|e| (v.number, e.into()))?;

					match r {
						Ok(r) => {
							log::info!(
						target: MOONBEAM_SUBMIT_LOG_TARGET,
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
						target: MOONBEAM_SUBMIT_LOG_TARGET,
						"Error submit verification |data owner:{:}|root_hash:{:}|request_hash: {:}, err: {:?}",
						v.data_owner,
						hex::encode(v.root_hash),
						hex::encode(v.request_hash),
						e
					);
							return Err((v.number, e.into()))
						},
					}
				}
			},
			Err(e) => return Err((v.number, e.into())),
		}
	}

	Ok(())
}
