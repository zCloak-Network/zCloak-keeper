use std::time::Duration;

use keeper_primitives::{
	monitor::{MonitorMetrics, MonitorSender},
	moonbeam::MOONBEAM_LOG_TARGET,
	ConfigInstance, Delay, Error, JsonParse, MqReceiver, MqSender, CHANNEL_LOG_TARGET,
};

use crate::U64;

use super::KeeperResult;

pub async fn task_scan(
	config: &ConfigInstance,
	msg_sender: &mut MqSender,
	mut start: U64,
	monitor_sender: MonitorSender,
) -> KeeperResult<()> {
	let mut tmp_start_cache = 0.into();

	loop {
		let maybe_best = config.moonbeam_client.best_number().await;
		let best = match maybe_best {
			Ok(b) => b,
			Err(e) => {
				log::error!(
						target: MOONBEAM_LOG_TARGET,
						"Fail to get latest block number in tasks moonbeam scan, after #{:?} scanned, err is {:?}",
						start,
						 e
					);
				// todo: make this more tolerant, e.g. retry N times first before throw and quit
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
				use tokio::time::{sleep, Duration};
				log::info!("sleep for scan block... current:{:}|best:{:}", start, latest);
				sleep(Duration::from_secs(keeper_primitives::moonbeam::MOONBEAM_BLOCK_DURATION))
					.await;
			}
		}

		// reset scan start point
		start = end;
	}
}

pub async fn task_submit(
	config: &ConfigInstance,
	msg_receiver: &mut MqReceiver,
	monitor_sender: MonitorSender,
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

		let res = super::submit_txs(
			&config.aggregator_contract,
			config.private_key,
			config.keeper_address,
			inputs,
		)
		.await;

		match res {
			Ok(_) => {},
			Err(e)  => {
				if cfg!(feature = "monitor") {
					let monitor_metrics = MonitorMetrics::new(
						MOONBEAM_LOG_TARGET.to_string(),
						e.0,
						e.1.into(),
						config.keeper_address,
					);
					monitor_sender.send(monitor_metrics).await;
				}
			}
		}

		// submit success or fail, both commit msg
		r.commit().map_err(|e| (None, e.into()))?;
	}

	Ok(())
}
