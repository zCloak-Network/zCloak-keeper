#![feature(async_closure)]
use secp256k1::SecretKey;
use std::ops::Add;
use web3::{
	signing::{Key, SecretKeyRef},
	types::{TransactionId, H256, U256},
};

use crate::task::RetryQueue;
use keeper_primitives::{
	moonbeam::{
		self, Events, Params, ProofEvent, IS_FINISHED, MAX_RETRY_TIMES, MOONBEAM_LISTENED_EVENT,
		MOONBEAM_RESUBMIT_LOG_TARGET, MOONBEAM_SCAN_LOG_TARGET, MOONBEAM_SCAN_SPAN,
		MOONBEAM_SUBMIT_LOG_TARGET, SUBMIT_STATUS_QUERY, SUBMIT_VERIFICATION,
	},
	Address, ConfigInstance, Contract, Http, MoonbeamClient, Result as KeeperResult, VerifyResult,
	Web3Options, U64,
};
pub use task::{create_retry_queue, task_resubmit, task_scan, task_submit, FatTx};

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

// only throw the error if fail to get `send_at` to construct the result
// that will be passed to the next task
pub async fn submit_txs(
	config: &ConfigInstance,
	contract: &Contract<Http>,
	keeper_pri: SecretKey,
	inputs: Vec<VerifyResult>,
	last_sent_tx: &mut FatTx,
) -> Result<Vec<FatTx>, (Option<U64>, moonbeam::Error)> {
	let key_ref = SecretKeyRef::new(&keeper_pri);
	let keeper_address = key_ref.address();

	// init a vec to hold info to pass to next task
	let mut result_for_next_task = vec![];

	for v in inputs {
		// TODO: read multiple times?
		// todo:throw error in production network
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

		// no need to submit
		if has_submitted || is_finished {
			continue
		}

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
		// todo: update
		let params = v1.get_submit_params();

		// pick a nonce, construct raw tx and send it onchain
		// todo: throw?
		let nonce = latest_nonce(
			Some(last_sent_tx.clone()),
			config,
			keeper_address,
			MOONBEAM_SUBMIT_LOG_TARGET,
		)
		.await
		.ok();
		let options = Web3Options::with(|options| options.nonce = nonce);
		// MUST get a value, otherwise throw it out
		let send_at = config
			.moonbeam_client
			.eth()
			.block_number()
			.await
			.map_err(|e| (None, e.into()))?;
		let tx_hash = construct_tx_and_send(contract, keeper_pri, options.clone(), params).await;
		// Ok(hash) -> Some(hash)
		// Err(_) => None and log error
		let tx_hash = match tx_hash {
			Ok(hash) => {
				log::info!(
					target: MOONBEAM_SUBMIT_LOG_TARGET,
					"[already submitted]|tx:{:}|data owner:{:}|root_hash:{:}|is_passed: {:}|attester: {:}",
					hash,
					v.data_owner,
					hex::encode(v.root_hash),
					v.is_passed,
					hex::encode(v.attester),
				);
				Some(hash)
			},
			Err(_) => {
				log::error!(
					target: MOONBEAM_SUBMIT_LOG_TARGET,
					"[failed to submit]|data owner:{:}|root_hash:{:}|is_passed: {:}|attester: {:}",
					v.data_owner,
					hex::encode(v.root_hash),
					v.is_passed,
					hex::encode(v.attester),
				);
				None
			},
		};

		let tx_info = (tx_hash, v.clone());
		// todo: record gas price?
		*last_sent_tx = FatTx { send_at, nonce, tx: tx_info.clone(), ..Default::default() };

		result_for_next_task.push(last_sent_tx.clone());
	}

	Ok(result_for_next_task)
}

// core logic to handle the resubmit task
// take item from the front of the list to check
// and push the latest submitted tx from the back
// will throw error if:
// - updating nonce fails
// - updating gas price fails
pub async fn resubmit_txs(
	config: &ConfigInstance,
	contract: &Contract<Http>,
	keeper_pri_optional: Option<SecretKey>,
	queue: RetryQueue,
) -> Result<(), (Option<U64>, moonbeam::Error)> {
	// if optional key is not set
	// just return to commit the msg in the channel
	let keeper_sec_key = match keeper_pri_optional {
		Some(sec_key) => sec_key,
		None => return Ok(()),
	};

	let key_ref = SecretKeyRef::new(&keeper_sec_key);
	let keeper_address = key_ref.address();

	let mut queue_guard = queue.lock().await;

	// check the tx hash from the front
	while let Some(mut item) = queue_guard.pop_front() {
		let is_included = match item.tx_info().0 {
			Some(hash) => {
				let maybe_tx_hash =
					config.moonbeam_client.eth().transaction(TransactionId::Hash(hash)).await;

				// if tx hash is included
				if let Ok(Some(tx)) = maybe_tx_hash {
					log::info!(
						target: MOONBEAM_RESUBMIT_LOG_TARGET,
						"[Already included] tx_hash [{:}] included in blockNumber: #{:?} which contains user address: {:} |request_hash: {:?}| root hash : {:} | isPassed: {}",
						tx.hash,
						tx.block_number,
						&item.tx_info().1.data_owner,
						hex::encode(&item.tx_info().1.request_hash),
						hex::encode(item.tx_info().1.root_hash),
						item.tx_info().1.is_passed
					);
					true
				} else {
					// not included yet or encounter error
					item.retry_times += 1;
					false
				}
			},

			None => {
				// no tx hash provided from last task
				false
			},
		};

		// if included, consume this item and continue to check next one
		if is_included {
			continue
		}

		// if tx not included and retry times <= max, push back the item
		// to the queue and break
		if item.retry_times <= MAX_RETRY_TIMES {
			queue_guard.push_front(item);
			break
		}

		// if the tx has been retried enough times(max_retry_times) still not included
		// or tx hash is not passed from the last task, re-construct and submit
		let (new_nonce, new_price) = {
			let last_sent_tx = queue_guard.back().map(|f| f.fat_tx.clone());
			// update nonce, will throw error
			let nonce =
				latest_nonce(last_sent_tx, &config, keeper_address, MOONBEAM_RESUBMIT_LOG_TARGET)
					.await
					.map_err(|e| (None, e.into()))?;

			// update gas
			// suggest gas price fetch, if error throw
			// throw error if gas_price fetching fails
			let suggested_gas_price =
				config.moonbeam_client.eth().gas_price().await.map_err(|e| (None, e.into()))?;
			let gas_price = {
				// todo: make 1.1 configurable!!
				let new_price = item.fat_tx.gas_price * 110 / 100;
				let new_price =
					if new_price > suggested_gas_price { new_price } else { suggested_gas_price };
				new_price
			};

			(nonce, gas_price)
		};

		let resubmit_strategy = Web3Options::with(|options| {
			options.nonce = Some(new_nonce);
			options.gas_price = Some(new_price);
		});

		let params = item.tx_info().1.get_submit_params();
		let send_at = config
			.moonbeam_client
			.eth()
			.block_number()
			.await
			.map_err(|e| (None, e.into()))?;
		let tx_hash =
			construct_tx_and_send(contract, keeper_sec_key, resubmit_strategy, params).await;
		// update the queue
		match tx_hash {
			Ok(hash) => {
				log::info!(
					target: MOONBEAM_RESUBMIT_LOG_TARGET,
					"[re submitted]|tx:{:}|data owner:{:}|root_hash:{:}|is_passed: {:}|attester: {:}",
					hash,
					item.tx_info().1.data_owner,
					hex::encode(item.tx_info().1.root_hash),
					item.tx_info().1.is_passed,
					hex::encode(item.tx_info().1.attester),
				);
				// update item

				// succeed to send, push it from the back to update the latest nonce
				queue_guard.push_back(item.clone());
			},

			Err(_) => {
				// fail to send, put it back from the front
				queue_guard.push_front(item.clone());
			},
		}

		item.update_after_resubmit(new_nonce, new_price, send_at, tx_hash.ok());
	}

	Ok(())
}

// pick a nonce to construct tx and send
// succeed if it returns Ok(tx_hash)
// if fail to get nonce, will throw error
pub async fn construct_tx_and_send(
	contract: &Contract<Http>,
	keeper_pri: SecretKey,
	// including updated nonce
	mut options: Web3Options,
	params: Params,
) -> Result<H256, moonbeam::Error> {
	// construct the send option, the must important thing is nonce.
	options.gas = Some(1000000_u128.into());

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
pub async fn latest_nonce(
	last_sent_tx: Option<FatTx>,
	config: &ConfigInstance,
	keeper_address: Address,
	log_target: &str,
) -> Result<U256, moonbeam::Error> {
	// best == last.send_at && nonce has value
	let nonce = match last_sent_tx {
		None => config
			.moonbeam_client
			.eth()
			.transaction_count(keeper_address, None)
			.await
			.map_err(|e| moonbeam::Error::Web3Error(e))?,
		Some(fat_tx) => {
			// pick last item in the queue, for the last item will hold the newest nonce.
			let best = config.moonbeam_client.best_number().await.map_err(|e| {
				log::error!(target: log_target, "[nonce] fail to get best, err is {:?}", e);
				e
			})?;

			// todo: use functional prpgraming way to rewrite
			if best == fat_tx.send_at && fat_tx.nonce.is_some() {
				fat_tx.nonce.unwrap() + U256::one()
			} else {
				// 1. best != last.send_at
				// 2. last.nonce is none
				config
					.moonbeam_client
					.eth()
					.transaction_count(keeper_address, None)
					.await
					.map_err(|e| moonbeam::Error::Web3Error(e))?
			}
		},
	};

	Ok(nonce)
}
