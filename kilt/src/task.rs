use std::time::Duration;

use keeper_primitives::{
	kilt::KILT_LOG_TARGET, ConfigInstance, Delay, Error, Hash, MqReceiver, MqSender, VerifyResult,
	U64,
};

pub async fn task_attestation(
	config: &ConfigInstance,
	msg_queue: (&mut MqSender, &mut MqReceiver),
) -> Result<(), (Option<U64>, Error)> {
	while let Ok(r) = msg_queue.1.recv_timeout(Delay::new(Duration::from_secs(1))).await {
		// while let Ok(events) = event_receiver.recv().await {
		let msg = match r {
			Some(a) => a,
			None => continue,
		};

		// parse verify result from str to VerifyResult
		let input_str = std::str::from_utf8(&*msg).expect("wrong format of msg into kilt task");
		let inputs: (Hash, Vec<VerifyResult>) = serde_json::from_str(input_str)
			.map_err(|e| {
				// log error
				log::error!(
					target: KILT_LOG_TARGET,
					"messages in task kilt wrongly parsed, {:?}",
					e
				);
			})
			.expect("fail to parse msg in kilt task");

		// the identifier for a batch of data
		let batch_id = inputs.0;
		log::info!(target: KILT_LOG_TARGET, "recv msg[{:}] in task3", hex::encode(batch_id));

		// have handled resoluble error inside filter
		let res = super::filter(&config.kilt_client, inputs.1).await.map_err(|e| (e.0, e.1))?;

		if !res.is_empty() {
			let res_str = serde_json::to_string(&(batch_id, res))
				.expect("outputs fail to parse in task kilt");
			let msg_to_send = res_str.as_bytes();
			// todo: handle?
			let _res = msg_queue.0.send(msg_to_send).await;
		}
		msg.commit().map_err(|e| (None, e.into()))?;
	}

	Ok(())
}
