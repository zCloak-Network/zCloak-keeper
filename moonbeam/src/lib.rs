
use secp256k1::SecretKey;
use std::{collections::LinkedList, ops::Add, sync::Mutex, thread::sleep, time::Duration};
use tokio::sync::MutexGuard;
use web3::{
	signing::{Key, SecretKeyRef},
	types::{TransactionId, H256, U256},
};

use crate::task::LocalReceiptQueue;
use keeper_primitives::{moonbeam::{
	self, Events, ProofEvent, IS_FINISHED, MOONBEAM_LISTENED_EVENT,
	MOONBEAM_RESUBMIT_LOG_TARGET, MOONBEAM_SCAN_LOG_TARGET, MOONBEAM_SCAN_SPAN,
	MOONBEAM_SUBMIT_LOG_TARGET, SUBMIT_STATUS_QUERY,
	SUBMIT_VERIFICATION,
}, Address, Bytes32, Config, ConfigInstance, Contract, Http, MoonbeamClient, Result as KeeperResult, VerifyResult, Web3Options, U64, Error};
pub use task::{create_receipt_queue, task_scan, task_submit, task_resubmit, LocalReceipt};

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

// (tx_hash, necessary info to construct tx params)
type TxHashAndInfo = (Option<H256>, VerifyResult);

pub async fn submit_txs(
	config: &ConfigInstance,
	contract: &Contract<Http>,
	keeper_pri: SecretKey,
	res: Vec<VerifyResult>,
	queue: LocalReceiptQueue,
) -> Result<Vec<TxHashAndInfo>, (Option<U64>, moonbeam::Error)> {
	let key_ref = SecretKeyRef::new(&keeper_pri);
	let keeper_address = key_ref.address();
	let mut queue_guard = queue.lock().await;
	let mut result_for_next_task = vec![];

	for v in res {
		// TODO: read multiple times?
		// if unable to get `has_submitted` result, then use false
		let has_submitted: bool = contract
			.query(
				SUBMIT_STATUS_QUERY,
				(keeper_address, v.data_owner, v.request_hash),
				None,
				Web3Options::default(),
				None,
			)
			.await
			.map_err(|e| {
				log::error!(
					target: MOONBEAM_SUBMIT_LOG_TARGET,
					"has_submiited query error: {:?}",
					&e
				);
				e
			})
			.unwrap_or_default();

		// if unable to get `is_finished` result, then use false
		let is_finished: bool = contract
			.query(IS_FINISHED, (v.data_owner, v.request_hash), None, Web3Options::default(), None)
			.await
			.map_err(|e| {
				log::error!(
					target: MOONBEAM_SUBMIT_LOG_TARGET,
					"is_finished query error: {:?}",
					&e
				);
				e
			})
			.unwrap_or_default();

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
			let v1 = v.clone();
			let params = (
				v1.data_owner,
				v1.request_hash,
				v1.c_type,
				v1.root_hash,
				v1.is_passed,
				v1.attester,
				v1.calc_output,
			);

			// pick a nonce, construct raw tx and send it onchain
			// todo: throw?
			let nonce = latest_nonce(&queue_guard, config, keeper_address)
				.await;
			let tx_hash =
				construct_tx_and_send(contract, keeper_pri, nonce, params).await;
			// for we do not know whether the tx will be packed in block, so we put related
			// information as `LocalReceipt` and push into the end of the queue.
			if let Ok(hash) = tx_hash {
				let send_at =
					config.moonbeam_client.best_number().await.map_err(|e| (None, e.into()))?;
				let local_receipt = LocalReceipt { send_at, nonce: nonce.unwrap_or_default(), tx_hash: hash };
				queue_guard.push_back(local_receipt);

				result_for_next_task.push((Some(hash), v.clone()));

				log::info!(
					target: MOONBEAM_SUBMIT_LOG_TARGET,
					"[already submitted]|tx:{:}|data owner:{:}|root_hash:{:}|is_passed: {:}|attester: {:}",
					hash,
					v.data_owner,
					hex::encode(v.root_hash),
					v.is_passed,
					hex::encode(v.attester),
				);
				log::info!(
					target: MOONBEAM_SUBMIT_LOG_TARGET,
					"[queue_info] nonce:{:}|queue_len:{:}",
					nonce.unwrap_or_default(),
					queue_guard.len(),
				);
			} else {
				// handle error, push the verifyresult into the message vec
				// which will be sent to the next task
				result_for_next_task.push((None, v));
			}

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

	Ok(result_for_next_task)
}

pub async fn resubmit_txs(
	config: &ConfigInstance,
	contract: &Contract<Http>,
	keeper_pri_optional: Option<SecretKey>,
	res: Vec<TxHashAndInfo>,
	queue: LocalReceiptQueue,
) -> Result<(), (Option<U64>, moonbeam::Error)> {
	// if optional key is not set
	// just return to commit the msg in the channel
	if keeper_pri_optional.is_none() {
		return Ok(())
	}
	let keeper_pri_optional = keeper_pri_optional.unwrap();
	// todo: struct the code, where to get the keeper address
	let key_ref = SecretKeyRef::new(&keeper_pri_optional);
	let keeper_address = key_ref.address();

	let mut queue_guard = queue.lock().await;

	for tx in res {
		match tx.0 {
			Some(hash) => {
				queue_guard.push_back(LocalReceipt::new_with_tx_hash(hash));
			},
			None => {
				// resubmit
				log::info!(
					target: MOONBEAM_RESUBMIT_LOG_TARGET,
					"Start Resubmitting: tx which contains user address: {:} |request_hash: {:}| root hash : {:} | isPassed: {}",
					tx.1.data_owner,
					hex::encode(tx.1.request_hash),
					hex::encode(tx.1.root_hash),
					tx.1.is_passed
				);

				// construct parameters for the contract call.
				let params = (
					tx.1.data_owner,
					tx.1.request_hash,
					tx.1.c_type,
					tx.1.root_hash,
					tx.1.is_passed,
					tx.1.attester,
					tx.1.calc_output,
				);

				// pick a nonce, construct raw tx and send it onchain
				// todo: throw?
				let nonce = latest_nonce(&queue_guard, config, keeper_address)
					.await;
				// tx_hash here must be a Ok value
				let tx_hash = loop {
					let hash = construct_tx_and_send(
						contract,
						keeper_pri_optional,
						nonce,
						params.clone(),
					)
					.await;
					if hash.is_ok() {
						break hash
					}

					// todo: make this configurable
					sleep(Duration::from_secs(1));
				};

				// must succeed
				let tx_hash = tx_hash.unwrap();
				// todo: unwrap with an error?
				let send_at =
					config.moonbeam_client.best_number().await.map_err(|e| (None, e.into()))?;
				let local_receipt = LocalReceipt { send_at, nonce: nonce.unwrap_or_default(), tx_hash: tx_hash.clone() };
				queue_guard.push_back(local_receipt);

				log::info!(
					target: MOONBEAM_RESUBMIT_LOG_TARGET,
					"[re submitted]|tx:{:}|data owner:{:}|root_hash:{:}|is_passed: {:}|attester: {:}",
					tx_hash,
					tx.1.data_owner,
					hex::encode(tx.1.root_hash),
					tx.1.is_passed,
					hex::encode(tx.1.attester),
				);

				log::debug!(
					target: MOONBEAM_RESUBMIT_LOG_TARGET,
					"[queue_info] queue detail:{:}",
					{
						use std::fmt::Write;
						let mut s = String::new();
						for i in queue_guard.iter() {
							write!(&mut s, "|{:?}", i).expect("fmt must be valid.");
						}
						s
					}
				);
			},
		}
	}

	Ok(())
}

// pick a nonce to construct tx and send
// succeed if it returns Ok(tx_hash)
// if fail to get nonce, will throw error
pub async fn construct_tx_and_send<'a>(
	contract: &Contract<Http>,
	keeper_pri: SecretKey,
	nonce: Option<U256>,
	params: (Address, Bytes32, Bytes32, Bytes32, bool, Bytes32, Vec<u128>),
) -> Result<H256, moonbeam::Error> {
	// construct the send option, the must important thing is nonce.
	let mut options = Web3Options::default();
	options.gas = Some(1000000_u128.into());

	match nonce {
		None => {},
		Some(nonce) => {
			options.nonce = Some(nonce);
		},
	};

	// todo: adjust gas price?
	// send tx for this contract call, and return tx_hash immediately.
	// todo: handle the send tx error
	contract
		.signed_call(SUBMIT_VERIFICATION, params, options, &keeper_pri)
		.await
		.map_err(|e| {
			log::error!(
						target: MOONBEAM_SUBMIT_LOG_TARGET,
						"[submit error] fail to submit: {:?}",
						e
					);
			e.into()
		})
}

// will throw error if failed to get nonce
pub async fn latest_nonce<'a>(
	queue_guard: &MutexGuard<'a, LinkedList<LocalReceipt>>,
	config: &ConfigInstance,
	keeper_address: Address,
) -> Option<U256> {
	// pick last item in the queue, for the last item will hold the newest nonce.
	let nonce = if let Some(last) = queue_guard.back() {
		// don't throw error out
		let best = config.moonbeam_client.best_number().await.unwrap_or_else(|e| {
			log::error!(
				target: MOONBEAM_SUBMIT_LOG_TARGET,
				"[nonce] fail to get best, err is {:?}",
				e
			);
			Default::default()
		});
		// TODO may need best hash
		if best == last.send_at {
			// it means the `send_at` block is same the current best, so we handle the nonce
			// in local +1 for next nonce.
			Some(last.nonce + U256::one())
		} else {
			// best != last.send_at or best is unable to get
			None
		}
	} else {
		None
	};

	// TODO use functional way to re-write this part.
	// throw out if we cannot retrieve nonce on chain
	let nonce = match nonce {
		Some(n) => Some(n),
		None => config
			.moonbeam_client
			.eth()
			.transaction_count(keeper_address, None)
			.await
			.ok()
	};
	nonce
}
