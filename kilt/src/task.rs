use std::time::Duration;
use keeper_primitives::{Delay, ConfigInstance, CHANNEL_LOG_TARGET, MqSender, MqReceiver, monitor::MonitorSender, JsonParse, EventResult};

pub async fn task_attestation(
    config: &ConfigInstance,
    msg_queue: (&mut MqSender, &mut MqReceiver),
    monitor_sender: MonitorSender
) {
    while let Ok(r) = msg_queue.1.recv_timeout(Delay::new(Duration::from_secs(1))).await {
        // while let Ok(events) = event_receiver.recv().await {
        let r = match r {
            Some(a) => a,
            None => continue,
        };
        log::info!(target: CHANNEL_LOG_TARGET, "recv msg in task3");
        // parse verify result from str to VerifyResult
        let inputs = serde_json::from_slice(&*r).expect("serde json error in tasks attestation");

        // have handled resoluble error inside filter
        let res = super::filter(&config.kilt_client, inputs).await.expect("Inner error in Kilt Attestation query");

        if !res.is_empty() {
            let message_to_send = serde_json::to_vec(&res);
            msg_queue.0
                .send(message_to_send.unwrap())
                .await
                .expect("[Task attestation] Fail to send msg to next tasks.");

            r.commit().expect("msg not commit in tasks attestation");
        }
    }
}