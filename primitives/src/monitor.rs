use tokio::sync::mpsc::error::SendError;
use super::{Address, U64};
use tokio::sync::mpsc::{Sender, Receiver};
use super::{Serialize, Deserialize};

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct MonitorMessage {
	// align with log target
	target: String,
	block_number: U64,
	// todo: structure this
	error: String,
	keeper_address: Address,
}

pub type MonitorSender = Sender<MonitorMessage>;
pub type MonitorReceiver = Receiver<MonitorMessage>;

impl MonitorMessage {
	pub async fn send(self, sender: MonitorSender) -> std::result::Result<(), SendError<Self>>{
		sender.send(self).await
	}

	// pub fn send_to_robot()
}



pub fn new_monitor_channel(buffer: usize) -> (MonitorSender, MonitorReceiver) {
	tokio::sync::mpsc::channel(buffer)
}




