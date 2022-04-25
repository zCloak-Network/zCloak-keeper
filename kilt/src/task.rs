use keeper_primitives::{
	monitor::MonitorSender, ConfigInstance, Delay, Error, MqReceiver, MqSender, CHANNEL_LOG_TARGET,
	U64,
};
use std::time::Duration;

pub async fn task_attestation(
	config: &ConfigInstance,
	msg_queue: (&mut MqSender, &mut MqReceiver),
) -> std::result::Result<(), (Option<U64>, Error)> {
	while let Ok(r) = msg_queue.1.recv_timeout(Delay::new(Duration::from_secs(1))).await {
		// while let Ok(events) = event_receiver.recv().await {
		let r = match r {
			Some(a) => a,
			None => continue,
		};
		log::info!(target: CHANNEL_LOG_TARGET, "recv msg in task3");
		// parse verify result from str to VerifyResult
		let inputs = serde_json::from_slice(&*r).map_err(|e| (None, e.into()))?;

		// have handled resoluble error inside filter
		let res = super::filter(&config.kilt_client, inputs).await.map_err(|e| (e.0, e.1))?;

		if !res.is_empty() {
			let message_to_send = serde_json::to_vec(&res);
			msg_queue.0.send(message_to_send.unwrap()).await.map_err(|e| (None, e.into()))?;
		}
		r.commit().map_err(|e| (None, e.into()))?;
	}

	Ok(())
}
