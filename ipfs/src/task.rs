use std::time::Duration;

use keeper_primitives::{
	ConfigInstance, Delay, Error, Events, JsonParse, MESSAGE_PARSE_LOG_TARGET, MqReceiver,
	MqSender, U64,
};

// todo: get block number in error return
pub async fn task_verify(
	config: &ConfigInstance,
	msg_queue: (&mut MqSender, &mut MqReceiver),
) -> std::result::Result<(), (Option<U64>, Error)> {
	while let Ok(events) = msg_queue.1.recv_timeout(Delay::new(Duration::from_secs(1))).await {
		let events = match events {
			Some(a) => a,
			None => continue,
		};

		// parse event from str to ProofEvent
		let inputs = Events::try_from_bytes(&*events);
		let inputs = match inputs {
			Ok(r) => r,
			Err(e) => {
				// log error
				log::error!(
					target: MESSAGE_PARSE_LOG_TARGET,
					"event messages in ipfs component wrongly parsed, {:?}",
					e
				);
				return Err((None, e.into()))
			},
		};

		let res = super::query_and_verify(&config.ipfs_client, inputs).await?;
		// not empty
		if res.is_some() {
			// todo : ugly hacking
			let start = res.clone().unwrap().first().unwrap().number;
			let status =
				msg_queue.0.send(serde_json::to_vec(&res).map_err(|e| (start, e.into()))?).await;

			match status {
				Ok(_) => {
					// delete events in channel after the events are successfully
					// transformed and pushed into
					events.commit().map_err(|e| (start, e.into()))?;
				},
				Err(e) => {
					log::error!("in task2 send to queue error:{:?}", e);
					return Err((None, e.into()))
				},
			}
		}
	}

	Ok(())
}
