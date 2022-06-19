use crate::U64;

use std::{collections::linked_list::LinkedList, sync::Arc};

use keeper_primitives::{
	monitor::{MonitorMetrics, MonitorSender},
	moonbeam::{MOONBEAM_SCAN_LOG_TARGET, MOONBEAM_SUBMIT_LOG_TARGET},
	ConfigInstance, Delay, Error, JsonParse, MqReceiver, MqSender, CHANNEL_LOG_TARGET,
};
use tokio::{
	sync::Mutex,
	time::{sleep, Duration},
};
use web3::types::{TransactionId, H256, U256};

use super::KeeperResult;

#[derive(Debug)]
pub struct LocalReceipt {
	pub send_at: U64,
	pub nonce: U256,
	pub tx_hash: H256,
}

pub type LocalReceiptQueue = Arc<Mutex<LinkedList<LocalReceipt>>>;

pub fn create_queue() -> LocalReceiptQueue {
	Arc::new(Mutex::new(LinkedList::<LocalReceipt>::new()))
}

pub async fn task_scan(
	config: &ConfigInstance,
	msg_sender: &mut MqSender,
	mut start: U64,
	_monitor_sender: MonitorSender,
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

const MAX_LOCAL_RECEIPT_QUEUE: usize = 100;

pub async fn task_submit(
	config: &ConfigInstance,
	msg_receiver: &mut MqReceiver,
	monitor_sender: MonitorSender,
	queue: LocalReceiptQueue,
) -> std::result::Result<(), (Option<U64>, Error)> {
	while let Ok(r) = msg_receiver.recv_timeout(Delay::new(Duration::from_secs(1))).await {
		// while let Ok(events) = event_receiver.recv().await {
		let r = match r {
			Some(a) => a,
			None => continue,
		};
		log::info!("recv msg in task4");
		// in theory, inputs wont be empty here
		let inputs = serde_json::from_slice(&*r).map_err(|e| (None, e.into()))?;

		// check queue before all execution process
		// 1. pop packed hash in the queue.
		let mut q = queue.lock().await;
		while let Some(item) = q.pop_front() {
			let r = config
				.moonbeam_client
				.eth()
				.transaction(TransactionId::Hash(item.tx_hash))
				.await
				.map_err(|e| (None, Error::MoonbeamError(e.into())))?;
			if r.is_some() {
				let i = q.pop_front().expect("item must exist here");
				log::info!(target: MOONBEAM_SUBMIT_LOG_TARGET, "[queue_info] pop item:{:?}", item);
			} else {
				// the tx has not be packed in blocks, so we push back front to the queue.
				q.push_front(item);
				break
			}
		}
		// 2. check queue length
		let len = q.len();
		// if current queue len is more than limit, for now, we just can return the Err for alert..
		// we may need to restart the node to re-send related transaction based of local receipt
		if len > MAX_LOCAL_RECEIPT_QUEUE {
			return Err((
				None,
				Error::ExceedQueueLen(len, q.front().expect("nothing").send_at.as_u64()),
			))
		}

		// 3. pick last item in the queue, for the last item will hold the newest nonce.
		let nonce = if let Some(last) = q.back() {
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
		drop(q);

		// 4. enter submit process.
		let res = super::submit_txs(
			config,
			&config.aggregator_contract,
			config.private_key,
			config.keeper_address,
			inputs,
			nonce,
			queue.clone(),
		)
		.await;

		match res {
			Ok(_) => {
				r.commit().map_err(|e| (None, e.into()))?;
			},
			Err(e) =>
				if cfg!(feature = "monitor") {
					let monitor_metrics = MonitorMetrics::new(
						MOONBEAM_SUBMIT_LOG_TARGET.to_string(),
						e.0,
						&e.1.into(),
						config.keeper_address,
					);
					monitor_sender.send(monitor_metrics).await;
				},
		}
	}

	Ok(())
}
