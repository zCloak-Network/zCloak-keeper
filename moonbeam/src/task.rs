use keeper_primitives::{
	Delay, JsonParse, MqReceiver, MqSender,
	CHANNEL_LOG_TARGET, TIMEOUT_DURATION, U64,
};
use std::sync::Arc;
use crate::metrics::{MoonbeamMetrics, MoonbeamMetricsExt};
use crate::funcs::{scan_events, submit_txs};
use keeper_primitives::monitor::{MonitorSender, NotifyingMessage};
use tokio::time::{sleep, timeout_at, Duration, Instant};
use keeper_primitives::traits::IpAddress;
use crate::MoonbeamResult;
use crate::types::{MOONBEAM_BLOCK_DURATION, MOONBEAM_SCAN_LOG_TARGET, Service};

pub async fn task_scan(
	service: &Service,
	msg_sender: &mut MqSender,
	mut start: U64,
	_monitor_sender: MonitorSender,
) -> MoonbeamResult<()> {
	let mut tmp_start_cache = 0.into();

	loop {
		let maybe_best =
			timeout_at(Instant::now() + TIMEOUT_DURATION, service.client.best_number())
				.await
				.map_err(|e| (None, e.into()))?;
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
			scan_events(start, best, &service.client, &service.client.proof_contract(&service.config.read_contract))
				.await?;

		if res.is_some() {
			// send result to channel
			// note: unwrap MUST succeed
			let output = res.unwrap().into_bytes().map_err(|e: serde_json::Error| (Some(start), e.into()))?;

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
			let latest = &service.client.best_number().await.unwrap_or_default();
			if start == *latest {
				// if current start is the best number, then sleep the block duration.
				log::info!("sleep for scan block... current:{:}|best:{:}", start, latest);
				sleep(Duration::from_secs(MOONBEAM_BLOCK_DURATION))
					.await;
			}
		}

		// reset scan start point
		start = end;
	}
}

pub async fn task_submit(
	service: &Service,
	msg_receiver: &mut MqReceiver,
	monitor_sender: MonitorSender,
) -> MoonbeamResult<()> {
	while let Ok(r) = msg_receiver.recv_timeout(Delay::new(Duration::from_secs(1))).await {
		let r = match r {
			Some(a) => a,
			None => continue,
		};
		log::info!("recv msg in task4");
		// in theory, inputs wont be empty here
		let inputs = serde_json::from_slice(&*r).map_err(|e| (None, e.into()))?;
		// todo: need a blocknumber here
		let res = submit_txs(
			//todo: move the func under Service
			&service.client.aggregator_contract(&service.config.write_contract),
			&service.private_key(),
			inputs,
		)
		.await;

		// todo: update metrics
		// service.metrics.is_some_and(|m| m.)
		// if service.metrics.is_some() {
		// 	service.metrics.report(|m| m.submitted_verify_transactions.inc());
		// }

		match res {
			Ok(_) => {
				r.commit().map_err(|e| (None, e.into()))?;
			},
			Err(e) => {},
			// todo: recover
			// 	if cfg!(feature = "monitor") {
			// 		let monitor_metrics = NotifyingMessage::new(
			// 			MOONBEAM_SUBMIT_LOG_TARGET.to_string(),
			// 			e.0,
			// 			&e.1.into(),
			// 			service.keeper_setting.keeper_address,
			// 			&service.client.ip_address(),
			// 		);
			// 		monitor_sender.send(monitor_metrics).await;
			// 	},
		}
	}

	Ok(())
}
