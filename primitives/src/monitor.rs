use super::Address;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct MonitorMessage {
	// align with log target
	target: String,
	// todo: stucture this
	error: String,
	timestamp: SystemTime,
	keeper_address: Address,
}
