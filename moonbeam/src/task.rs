use crate::{TxHashAndInfo, U64};

use std::{collections::linked_list::LinkedList, sync::Arc};

use keeper_primitives::{
	monitor::{MonitorMetrics, MonitorSender},
	moonbeam::{
		MAX_RETRY_TIMES, MOONBEAM_RESUBMIT_LOG_TARGET, MOONBEAM_SCAN_LOG_TARGET,
		MOONBEAM_SUBMIT_LOG_TARGET, QUEUE_EXPIRE_DURATION, RESUBMIT_INTERVAL,
	},
	ConfigInstance, Delay, Error, JsonParse, MqReceiver, MqSender, VerifyResult,
	CHANNEL_LOG_TARGET,
};
use tokio::{
	sync::Mutex,
	time::{sleep, Duration},
};
use web3::types::{TransactionId, H256, U256};

use super::KeeperResult;

#[derive(Default, Debug, Clone)]
pub struct LocalSentTx {
	pub send_at: Option<U64>,
	pub nonce: Option<U256>,
	pub tx_hash: H256,
}

impl LocalSentTx {
	pub(crate) fn new_with_tx_hash(tx_hash: H256) -> Self {
		LocalSentTx { tx_hash, ..Default::default() }
	}
}

pub type LocalSentQueue = Arc<Mutex<LinkedList<LocalSentTx>>>;

pub fn create_local_sent_queue() -> LocalSentQueue {
	Arc::new(Mutex::new(LinkedList::<LocalSentTx>::new()))
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
	last_sent_tx: &mut LocalSentTx,
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
	queue: LocalSentQueue,
) -> std::result::Result<(), (Option<U64>, Error)> {
	while let Ok(r) = msg_receiver.recv_timeout(Delay::new(Duration::from_secs(1))).await {
		// while let Ok(events) = event_receiver.recv().await {
		let r = match r {
			Some(a) => a,
			None => continue,
		};
		// false - first submit
		// true - resubmit
		// txs are not handled in the first submit stage, all troubled txs will flow to
		// the resubmit stage/task and do necessary retries.
		log::info!("recv msg in task5");
		// in theory, inputs wont be empty here
		let inputs = serde_json::from_slice(&*r).map_err(|e| (None, e.into()))?;

		// check queue before all execution process
		// 1. pop packed hash in the queue.
		let mut q = queue.lock().await;

		// wait RESUBMIT_INTERVAL secs to avoid resubmit a tx that has been already finished
		sleep(Duration::from_secs(RESUBMIT_INTERVAL)).await;

		while let Some(item) = q.pop_front() {
			let mut i = 0;
			let res = loop {
				let r = config
					.moonbeam_client
					.eth()
					.transaction(TransactionId::Hash(item.tx_hash))
					.await;

				if r.is_ok() || i == MAX_RETRY_TIMES {
					break r
				}
				i = i + 1;
				// todo use a variable to represent 1s
				sleep(Duration::from_secs(1)).await;
			};

			// if the tx is included or expired
			// todo: expiration is not the best way to handle unconfirmed tx, change it later
			if res.is_ok() {
				log::info!(
					target: MOONBEAM_RESUBMIT_LOG_TARGET,
					"[queue_info] pop item:{:?}",
					item
				);
			} else {
				// the tx has not be packed in blocks, so we push back front to the queue.
				log::error!(
					target: MOONBEAM_RESUBMIT_LOG_TARGET,
					"[queue_info] pop item with error:{:?}",
					item
				);
			}
		}
		// 2. check queue length
		let len = q.len();
		// if current queue len is more than limit, for now, we just can return the Err for alert..
		// we may need to restart the node to re-send related transaction based of local receipt
		if len > MAX_LOCAL_RECEIPT_QUEUE {
			return Err((None, Error::ExceedQueueLen(len, q.front().expect("nothing").send_at)))
		}
		drop(q);

		// enter submit process.
		// re-consume the same data if error thrown
		let res = super::resubmit_txs(
			config,
			&config.aggregator_contract,
			config.private_key_optional,
			inputs,
			// nonce,
			queue.clone(),
		)
		.await;

		match res {
			Ok(_) => {
				r.commit().map_err(|e| (None, e.into()))?;
			},
			Err(e) => {
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
			},
		}
	}

	Ok(())
}
