use super::*;
use super::types::Service;
use std::time::Duration;
use keeper_primitives::{
	Delay, Events, JsonParse, MqReceiver, MqSender, CHANNEL_LOG_TARGET,
	MESSAGE_PARSE_LOG_TARGET, U64,
};
use crate::funcs::query_and_verify;

// todo: get block number in error return
pub async fn task_verify(
	service: &Service,
	msg_queue: (&mut MqSender, &mut MqReceiver),
) -> Result<()> {
	while let Ok(events) = msg_queue.1.recv_timeout(Delay::new(Duration::from_secs(1))).await {
		let events = match events {
			Some(a) => a,
			None => continue,
		};

		log::info!(target: CHANNEL_LOG_TARGET, "recv msg in task2");

		// parse event from str to ProofEvent
		let inputs: std::result::Result<Events, serde_json::Error> = Events::try_from_bytes(&*events);
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

		let res = query_and_verify(&service.client, inputs).await?;
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
