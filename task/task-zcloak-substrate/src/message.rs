use lifeline::Message;
use postage::broadcast;
use serde::{Deserialize, Serialize};

use crate::bus::ZcloakTaskBus;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ZcloakTaskMessage {
	TaskEvent,
}

impl Message<ZcloakTaskBus> for ZcloakTaskMessage {
	type Channel = broadcast::Sender<Self>;
}
