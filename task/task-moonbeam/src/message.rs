use lifeline::Message;
use postage::broadcast;
use serde::{Deserialize, Serialize};

use crate::bus::MoonbeamTaskBus;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MoonbeamTaskMessage {
	TaskEvent,
}

impl Message<MoonbeamTaskBus> for MoonbeamTaskMessage {
	type Channel = broadcast::Sender<Self>;
}
