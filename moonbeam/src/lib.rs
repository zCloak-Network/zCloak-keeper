use secp256k1::SecretKey;
use web3::types::U256;

use crate::task::LocalReceiptQueue;
use keeper_primitives::{
	moonbeam::{
		self, Events, ProofEvent, IS_FINISHED, MOONBEAM_LISTENED_EVENT, MOONBEAM_SCAN_LOG_TARGET,
		MOONBEAM_SCAN_SPAN, MOONBEAM_SUBMIT_LOG_TARGET, MOONBEAM_TRANSACTION_CONFIRMATIONS,
		SUBMIT_STATUS_QUERY, SUBMIT_VERIFICATION,
	},
	Address, ConfigInstance, Contract, Http, MoonbeamClient, Result as KeeperResult, VerifyResult,
	Web3Options, U64,
};
pub use task::{create_queue, task_scan, task_submit, LocalReceipt};

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
	let r = moonbeam::utils::events::<_, ProofEvent>(
		client.eth(),
		proof_contract,
		MOONBEAM_LISTENED_EVENT,
		Some(start),
		Some(end),
	)
	.await;

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
	config: &ConfigInstance,
	contract: &Contract<Http>,
	keeper_pri: SecretKey,
	keeper_address: Address,
	res: Vec<VerifyResult>,
	queue: LocalReceiptQueue,
) -> Result<(), (Option<U64>, moonbeam::Error)> {
	let mut queue_guard = queue.lock().await;
	for v in res {
		// 3. pick last item in the queue, for the last item will hold the newest nonce.
		let nonce = if let Some(last) = queue_guard.back() {
			let best = config.moonbeam_client.best_number().await.map_err(|e| (None, e.into()))?;
			// TODO may need best hash
			if best == last.send_at {
				// it means the `send_at` block is same the current best, so we handle the nonce in
				// local +1 for next nonce.
				Some(last.nonce + U256::one())
			} else {
				None
			}
		} else {
			None
		};

		// TODO: read multiple times?
		let has_submitted: bool = contract
			.query(
				SUBMIT_STATUS_QUERY,
				(keeper_address, v.data_owner, v.request_hash),
				None,
				Web3Options::default(),
				None,
			)
			.await
			.map_err(|e| (v.number, e.into()))?;

		let is_finished: bool = contract
			.query(IS_FINISHED, (v.data_owner, v.request_hash), None, Web3Options::default(), None)
			.await
			.map_err(|e| (v.number, e.into()))?;

		log::info!(
			target: MOONBEAM_SUBMIT_LOG_TARGET,
			"record: block number: {:?} | request_hash: {:}| root hash : {:}| hasSubmitted is {}, isFinished result is {:}",
			v.number,
			hex::encode(v.request_hash),
			hex::encode(v.root_hash),
			has_submitted,
			is_finished
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

			// construct parameters for the contract call.
			let params = (
				v.data_owner,
				v.request_hash,
				v.c_type,
				v.root_hash,
				v.is_passed,
				v.attester,
				v.calc_output,
			);

			// TODO use functional way to re-write this part.
			let send_at =
				config.moonbeam_client.best_number().await.map_err(|e| (None, e.into()))?;
			let nonce = match nonce {
				Some(n) => n,
				None => config
					.moonbeam_client
					.eth()
					.transaction_count(config.keeper_address, None)
					.await
					.map_err(|e| (v.number, e.into()))?,
			};
			// construct the send option, the must important thing is nonce.
			let mut options = Web3Options::default();
			options.nonce = Some(nonce);
			options.gas = Some(1000000_u128.into());
			// send tx for this contract call, and return tx_hash immediately.
			let tx_hash = contract
				.signed_call(SUBMIT_VERIFICATION, params, options, &keeper_pri)
				.await
				.map_err(|e| (v.number, e.into()))?;
			// for we do not know whether the tx will be packed in block, so we put related
			// information as `LocalReceipt` and push into the end of the queue.
			let local_receipt = LocalReceipt { send_at, nonce, tx_hash };
			queue_guard.push_back(local_receipt);

			log::info!(
				target: MOONBEAM_SUBMIT_LOG_TARGET,
				"submit verification|tx:{:}|data owner:{:}|root_hash:{:}|is_passed: {:}|attester: {:}",
				tx_hash,
				v.data_owner,
				hex::encode(v.root_hash),
				v.is_passed,
				hex::encode(v.attester),
			);
			log::info!(
				target: MOONBEAM_SUBMIT_LOG_TARGET,
				"[queue_info] nonce:{:}|queue_len:{:}",
				nonce,
				queue_guard.len(),
			);
			log::debug!(target: MOONBEAM_SUBMIT_LOG_TARGET, "[queue_info] queue detail:{:}", {
				use std::fmt::Write;
				let mut s = String::new();
				for i in queue_guard.iter() {
					write!(&mut s, "|{:?}", i).expect("fmt must be valid.");
				}
				s
			});
		}
	}

	Ok(())
}
