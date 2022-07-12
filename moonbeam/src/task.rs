use crate::{TxHashAndInfo, U64};

use std::{collections::linked_list::LinkedList, sync::Arc};

use keeper_primitives::{
	monitor::{MonitorMetrics, MonitorSender},
	moonbeam::{
		MOONBEAM_RESUBMIT_LOG_TARGET, MOONBEAM_SCAN_LOG_TARGET, MOONBEAM_SUBMIT_LOG_TARGET,
		RESUBMIT_INTERVAL,
	},
	ConfigInstance, Delay, Deserialize, Error, JsonParse, MqReceiver, MqSender, Serialize,
	CHANNEL_LOG_TARGET,
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
			let output = res.unwrap().into_bytes().map_err(|e| (Some(start), e.into()))?;

			let status = msg_sender.send(output).await;
			if let Err(e) = status {
				log::error!(
					target: CHANNEL_LOG_TARGET,
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

const MAX_LOCAL_RECEIPT_QUEUE: usize = 200;

pub async fn task_submit(
	config: &ConfigInstance,
	msg_queue: (&mut MqSender, &mut MqReceiver),
	last_sent_tx: &mut FatTx,
) -> std::result::Result<(), (Option<U64>, Error)> {
	while let Ok(r) = msg_queue.1.recv_timeout(Delay::new(Duration::from_secs(2))).await {
		// while let Ok(events) = event_receiver.recv().await {
		let r = match r {
			Some(a) => a,
			None => continue,
		};

		log::info!("recv msg in task4");
		// in theory, inputs wont be empty here
		let inputs = serde_json::from_slice(&*r).map_err(|e| (None, e.into()))?;

		// enter submit process.
		let res = super::submit_txs(
			config,
			&config.aggregator_contract,
			config.private_key,
			inputs,
			// nonce,
			last_sent_tx,
		)
		.await;

		// res must be Ok
		if res.is_ok() {
			let res = res.unwrap();
			let status =
				msg_queue.0.send(serde_json::to_vec(&res).map_err(|e| (None, e.into()))?).await;

			match status {
				Ok(_) => {
					r.commit().map_err(|e| (None, e.into()))?;
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
	monitor_sender: MonitorSender,
	queue: RetryQueue,
	local_last_sent_at: &mut U64,
) -> Result<(), (Option<U64>, Error)> {
	while let Ok(r) = msg_receiver.recv_timeout(Delay::new(Duration::from_secs(1))).await {
		let r = match r {
			Some(a) => {
				let mut q = queue.lock().await;

				// wait RESUBMIT_INTERVAL secs to avoid resubmit a tx that has been already finished
				sleep(Duration::from_secs(RESUBMIT_INTERVAL)).await;

				// push the latest sent transactions to the back of the queue
				if a.len() > 0 {
					log::info!("recv msg in task5");
					// in theory, inputs wont be empty here
					let inputs: Vec<FatTx> =
						serde_json::from_slice(&*a).map_err(|e| (None, e.into()))?;

					// must be true because r is not empty and deserializing succeeds
					if let Some(last) = inputs.last() {
						let last_sent_at_from_outer = last.send_at;
						// if last_sent_from_outer <= local_last_sent, then skip updating the queue
						// because `resubmit_txs` will throw error and re-consume the data from the
						// channel
						if &last_sent_at_from_outer > local_last_sent_at {
							for tx_info in inputs {
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
						if cfg!(feature = "monitor") {
							let monitor_metrics = MonitorMetrics::new(
								MOONBEAM_SUBMIT_LOG_TARGET.to_string(),
								e.0,
								&e.1.into(),
								config.name.to_string(),
							);
							monitor_sender.send(monitor_metrics).await;
						}
					},
				}
			},
			None => {
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

				if let Err(e) = res {
					log::error!(target: MOONBEAM_SUBMIT_LOG_TARGET, "submit_txs error: {:?}", &e);
					if cfg!(feature = "monitor") {
						let monitor_metrics = MonitorMetrics::new(
							MOONBEAM_SUBMIT_LOG_TARGET.to_string(),
							e.0,
							&e.1.into(),
							config.name.to_string(),
						);
						monitor_sender.send(monitor_metrics).await;
					}
				}
			},
		};
	}

	Ok(())
}
