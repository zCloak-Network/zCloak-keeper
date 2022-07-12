use crate::{TxHashAndInfo, U64};

use codec::Encode;
use std::{collections::linked_list::LinkedList, sync::Arc};

use keeper_primitives::{
	moonbeam::{
		MOONBEAM_RESUBMIT_LOG_TARGET, MOONBEAM_SCAN_LOG_TARGET, MOONBEAM_SUBMIT_LOG_TARGET,
		RESUBMIT_INTERVAL,
	},
	ConfigInstance, Delay, Deserialize, Error, Hash, MqReceiver, MqSender, Serialize, VerifyResult,
};
use tokio::{
	sync::Mutex,
	time::{sleep, Duration},
};
use web3::types::{H256, U256};

use super::KeeperResult;

// fat tx contains the necessary info for constructing new tx
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FatTx {
	pub send_at: U64,
	pub gas_price: U256,
	pub nonce: Option<U256>,
	pub tx: TxHashAndInfo,
}

impl FatTx {
	pub(crate) fn new_with_tx_info(tx: TxHashAndInfo) -> Self {
		FatTx { tx, ..Default::default() }
	}
}

#[derive(Default, Debug, Clone)]
pub struct RetryTx {
	pub fat_tx: FatTx,
	pub retry_times: u8,
}

impl RetryTx {
	pub(crate) fn tx_info(&self) -> TxHashAndInfo {
		self.fat_tx.tx.clone()
	}

	// clear retry_times, update nonce, price, send_at and tx_hash
	pub(crate) fn update_after_resubmit(
		&mut self,
		nonce: U256,
		gas_price: U256,
		send_at: U64,
		tx_hash: Option<H256>,
	) {
		self.retry_times = 0;
		self.fat_tx.tx.0 = tx_hash;
		self.fat_tx.nonce = Some(nonce);
		self.fat_tx.gas_price = gas_price;
		self.fat_tx.send_at = send_at;
	}
}

pub type RetryQueue = Arc<Mutex<LinkedList<RetryTx>>>;

pub fn create_retry_queue() -> RetryQueue {
	Arc::new(Mutex::new(LinkedList::<RetryTx>::new()))
}

pub async fn task_scan(
	config: &ConfigInstance,
	msg_sender: &mut MqSender,
	mut start: U64,
) -> KeeperResult<()> {
	let mut tmp_start_cache = 0.into();

	loop {
		let maybe_best = config.moonbeam_client.best_number().await;
		let best = match maybe_best {
			Ok(b) => b,
			Err(e) => {
				log::error!(
						target: MOONBEAM_SCAN_LOG_TARGET,
						"Fail to get latest block number in tasks moonbeam scan, after #{:?} scanned, err is {:?}",
						start,
						 e
					);
				return Err((None, e.into()))
			},
		};

		// local network check
		// only work if the chain is frozen
		if (start == tmp_start_cache) && (start == best) {
			// do nothing here
			continue
		}

		// only throw err if event parse error
		// todo: could return and throw error instead of expect
		let (res, end) =
			super::scan_events(start, best, &config.moonbeam_client, &config.proof_contract)
				.await?;

		if res.is_some() {
			// send result to channel
			// unwrap MUST succeed
			let output =
				serde_json::to_string(&res.unwrap()).expect("output fail to parse in task scan");
			let msg_to_send = output.as_bytes();

			let status = msg_sender.send(msg_to_send).await;
			if let Err(e) = status {
				log::error!(
					target: MOONBEAM_SCAN_LOG_TARGET,
					"Fail to write data in block from: #{:?} into event channel file",
					start,
				);
				return Err((Some(start), e.into()))
			}
			// After the proofevent list successfully sent to task2
			// reset the tmp_start_cache
			tmp_start_cache = end;
		} else {
			let latest = &config.moonbeam_client.best_number().await.unwrap_or_default();
			if start == *latest {
				// if current start is the best number, then sleep the block duration.
				log::info!("sleep for scan block... current:{:}|best:{:}", start, latest);
				sleep(Duration::from_secs(keeper_primitives::moonbeam::MOONBEAM_BLOCK_DURATION))
					.await;
			}
		}

		// reset scan start point
		start = end;
	}
}

const MAX_RETRY_QUEUE_LEN: usize = 200;

pub async fn task_submit(
	config: &ConfigInstance,
	msg_queue: (&mut MqSender, &mut MqReceiver),
	last_sent_tx: &mut FatTx,
) -> Result<(), (Option<U64>, Error)> {
	while let Ok(r) = msg_queue.1.recv_timeout(Delay::new(Duration::from_secs(2))).await {
		// while let Ok(events) = event_receiver.recv().await {
		let msg = match r {
			Some(a) => a,
			None => continue,
		};

		let input_str = std::str::from_utf8(&*msg).expect("wrong format of msg into submit task");
		let inputs: (Hash, Vec<VerifyResult>) = serde_json::from_str(input_str)
			.map_err(|e| {
				// log error
				log::error!(
					target: MOONBEAM_SUBMIT_LOG_TARGET,
					"messages in task submit wrongly parsed, {:?}",
					e
				);
			})
			.expect("fail to parse msg in task submit");

		// the identifier for a batch of data
		let batch_id = inputs.0;
		log::info!("recv msg[{:}] in task4", hex::encode(batch_id));

		// enter submit process.
		let res = super::submit_txs(
			config,
			&config.aggregator_contract,
			config.private_key,
			inputs.1,
			// nonce,
			last_sent_tx,
		)
		.await;

		// res must be Ok
		if res.is_ok() {
			let output = serde_json::to_string(&(batch_id, res.unwrap()))
				.expect("output fail to parse in task submit");
			let msg_to_send = output.as_bytes();
			let status = msg_queue.0.send(msg_to_send).await;

			match status {
				Ok(_) => {
					msg.commit().map_err(|e| (None, e.into()))?;
				},
				Err(e) => {
					log::error!(target: MOONBEAM_SUBMIT_LOG_TARGET, "submit_txs error: {:?}", &e);
				},
			};
		}
	}

	Ok(())
}

pub async fn task_resubmit(
	config: &ConfigInstance,
	msg_receiver: &mut MqReceiver,
	queue: RetryQueue,
	local_last_sent_at: &mut U64,
) -> Result<(), (Option<U64>, Error)> {
	while let Ok(r) = msg_receiver.recv_timeout(Delay::new(Duration::from_secs(1))).await {
		// todo: change the architect to remove duplicated code
		match r {
			Some(a) => {
				let mut q = queue.lock().await;

				// wait RESUBMIT_INTERVAL secs to avoid resubmit a tx that has been already finished
				sleep(Duration::from_secs(RESUBMIT_INTERVAL)).await;

				// push the latest sent transactions to the back of the queue
				if a.len() > 0 {
					let input_str =
						std::str::from_utf8(&*a).expect("wrong format of msg into submit task");
					let inputs: (Hash, Vec<FatTx>) = serde_json::from_str(input_str)
						.map_err(|e| {
							// log error
							log::error!(
								target: MOONBEAM_RESUBMIT_LOG_TARGET,
								"messages in task resubmit wrongly parsed, {:?}",
								e
							);
						})
						.expect("fail to parse msg in task submit");

					let batch_id = inputs.0;
					log::info!("recv msg[{:}] in task5", hex::encode(batch_id));

					// in theory, inputs wont be empty here
					// must be true because r is not empty and deserializing succeeds
					if let Some(last) = inputs.1.last() {
						let last_sent_at_from_outer = last.send_at;
						// if last_sent_from_outer <= local_last_sent, then skip updating the queue
						// because `resubmit_txs` will throw error and re-consume the data from the
						// channel
						if &last_sent_at_from_outer > local_last_sent_at {
							for tx_info in inputs.1 {
								// discard the option and sent_at received from the last task
								// because task resubmit use a new private key
								q.push_back(RetryTx {
									fat_tx: FatTx::new_with_tx_info(tx_info.tx),
									..Default::default()
								});
								*local_last_sent_at = tx_info.send_at;
							}
						}

						log::debug!(
							target: MOONBEAM_RESUBMIT_LOG_TARGET,
							"[queue_info] queue detail:{:}",
							{
								use std::fmt::Write;
								let mut s = String::new();
								for i in q.iter() {
									write!(&mut s, "|{:?}", i).expect("fmt must be valid.");
								}
								s
							}
						);

						drop(q);
					};
				}

				// enter resubmit process.
				// will consume the queue from the front
				// will throw error if:
				// - updating nonce fails
				// - updating gas price fails
				let res = super::resubmit_txs(
					config,
					&config.aggregator_contract,
					config.private_key_optional,
					queue.clone(),
				)
				.await;

				match res {
					Ok(_) => {
						a.commit().map_err(|e| (None, e.into()))?;
					},
					Err(e) => {
						log::error!(
							target: MOONBEAM_SUBMIT_LOG_TARGET,
							"submit_txs error: {:?}",
							&e
						);
					},
				}
			},
			None => {
				// enter resubmit process.
				// will consume the queue from the front
				// will throw error if:
				// - updating nonce fails
				// - updating gas price fails
				super::resubmit_txs(
					config,
					&config.aggregator_contract,
					config.private_key_optional,
					queue.clone(),
				)
				.await
				.map_err(|e| {
					log::error!(target: MOONBEAM_SUBMIT_LOG_TARGET, "submit_txs error: {:?}", &e);
					(e.0, e.1.into())
				})?;
			},
		};
	}

	Ok(())
}
