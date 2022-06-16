use super::Address;
use std::net::IpAddr;

#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct KeeperSetting {
	pub address: Address,
	pub ip_address: Option<IpAddr>,
}

impl KeeperSetting {
	pub async fn new(address: Address) -> Self {
		let ip_address = public_ip::addr().await;
		Self { address, ip_address }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn keeper_setting_should_work() {
		let setting = KeeperSetting::new(Address::default()).await;
		assert!(setting.ip_address.is_some());
	}
}
