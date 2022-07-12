use std::time::Duration;

use keeper_primitives::{
	ipfs::IPFS_LOG_TARGET, ConfigInstance, Delay, Error, Events, Hash, MqReceiver, MqSender,
	MESSAGE_PARSE_LOG_TARGET, U64,
};

// todo: get block number in error return
pub async fn task_verify(
	config: &ConfigInstance,
	msg_queue: (&mut MqSender, &mut MqReceiver),
) -> std::result::Result<(), (Option<U64>, Error)> {
	while let Ok(inputs) = msg_queue.1.recv_timeout(Delay::new(Duration::from_secs(1))).await {
		let msg = match inputs {
			Some(a) => a,
			None => continue,
		};

		// parse event from str to ProofEvent
		let input_str = std::str::from_utf8(&*msg).expect("wrong format of msg into ipfs task");
		let inputs: (Hash, Events) = serde_json::from_str(input_str)
			.map_err(|e| {
				// log error
				log::error!(
					target: MESSAGE_PARSE_LOG_TARGET,
					"event messages in ipfs component wrongly parsed, {:?}",
					e
				);
			})
			.expect("fail to parse msg in ipfs task");

		let batch_id = inputs.0;
		log::info!(target: IPFS_LOG_TARGET, "recv msg[{:}] in task2", hex::encode(batch_id));

		let res = super::query_and_verify(&config.ipfs_client, inputs.1).await?;
		// not empty
		if res.is_some() {
			// todo : ugly hacking
			let start = res.clone().unwrap().first().unwrap().number;
			let res_str = serde_json::to_string(&(batch_id, res.unwrap()))
				.expect("outputs fail to parse in task ipfs");
			let msg_to_send = res_str.as_bytes();
			let status = msg_queue.0.send(msg_to_send).await;

			match status {
				Ok(_) => {
					// delete events in channel after the events are successfully
					// transformed and pushed into
					msg.commit().map_err(|e| (start, e.into()))?;
				},
				Err(e) => {
					log::error!("in task2 send to queue error:{:?}", e);
				},
			}
		}
	}

	Ok(())
}
