use std::time::Duration;
use keeper_primitives::{
    Delay,
    MESSAGE_PARSE_LOG_TARGET,
    ConfigInstance, MqSender, MqReceiver, monitor::MonitorSender, JsonParse, EventResult};

pub async fn task_verify(
    config: &ConfigInstance,
    msg_queue: (&mut MqSender, &mut MqReceiver),
    monitor_sender: MonitorSender
) {
    while let Ok(events) = msg_queue.1.recv_timeout(Delay::new(Duration::from_secs(1))).await
    {
        let events = match events {
            Some(a) => a,
            None => continue,
        };

        // parse event from str to ProofEvent
        let inputs = EventResult::try_from_bytes(&*events);
        let inputs = match inputs {
            Ok(r) => r,
            Err(e) => {
                // log error
                log::error!(
						target: MESSAGE_PARSE_LOG_TARGET,
						"event messages in ipfs component wrongly parsed, {:?}",
						e
					);
                continue
            },
        };

        let r = super::query_and_verify(&config.ipfs_client, inputs).await;
        let res = match r {
            Ok(v) => v,
            Err(e) => {
                log::error!(
						// TODO: log target?
						"[IPFS_AND_VERIFY] encounter error: {:?}",
						e
					);
                continue
            },
        };
        let status = msg_queue.0.send(serde_json::to_vec(&res).unwrap()).await;

        match status {
            Ok(_) => {
                // delete events in channel after the events are successfully
                // transformed and pushed into
                events.commit().expect("not commit in tasks ipfs_and_verify");
            },
            Err(e) => {
                log::error!("in task2 send to queue error:{:?}", e);
                continue
            },
        }
    }
}