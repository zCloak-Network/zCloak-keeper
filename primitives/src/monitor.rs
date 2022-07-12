use std::collections::HashMap;

use reqwest::Client;
use strfmt::Format;
use tokio::{
	sync::mpsc::{Receiver, Sender},
	time::Duration,
};

use super::{Deserialize, Serialize, U64};

const TIME_OUT: Duration = Duration::from_secs(5);

#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct MonitorConfig {
	pub bot_url: String,
}

// todo: structure monitor message send

#[derive(Debug)]
pub struct MonitorMetrics {
	// align with log target
	target: String,
	block_number: Option<U64>,
	error_msg: String,
	keeper_name: String,
}

pub type MonitorSender = Sender<MonitorMetrics>;
pub type MonitorReceiver = Receiver<MonitorMetrics>;

pub type KeywordReplace = HashMap<String, String>;

impl MonitorMetrics {
	pub fn new(
		target: String,
		block_number: Option<U64>,
		error: &super::Error,
		keeper_name: String,
	) -> Self {
		let error_msg = format!("{:?}", error);
		Self { target, block_number, error_msg, keeper_name }
	}

	pub fn new_with_target_and_error(
		target: String,
		error_with_number: &(Option<U64>, super::Error),
		name: String,
	) -> Self {
		Self::new(target, error_with_number.0, &error_with_number.1, name)
	}

	pub fn monitor_keywords(&self) -> KeywordReplace {
		let mut map = HashMap::new();
		// todo: config key
		map.insert("level".to_owned(), self.target.clone());
		map.insert("BlockNumber".to_owned(), self.get_block());
		map.insert("error".to_owned(), self.error_msg.clone());
		map.insert("KeeperName".to_owned(), self.keeper_name.clone());
		map
	}

	pub fn get_block(&self) -> String {
		match self.block_number {
			Some(n) => n.as_u64().to_string(),
			None => "None".to_owned(),
		}
	}

	pub fn message(&self) -> Result<String> {
		let replace = self.monitor_keywords();
		let template = include_str!("../res/monitor_template");

		Ok(template.format(&replace)?)
	}
}

pub fn new_monitor_channel(buffer: usize) -> (MonitorSender, MonitorReceiver) {
	tokio::sync::mpsc::channel(buffer)
}

pub async fn alert(bot_url: &str, body: String) -> Result<()> {
	let client = Client::builder().connect_timeout(TIME_OUT).build()?;
	client.post(bot_url).body(body).send().await?;
	Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("POST monitor bot error, reason: {0}")]
	HttpError(#[from] reqwest::Error),
	#[error("Monitor message pack error, err: {0}")]
	TemplateFormatError(#[from] strfmt::FmtError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use crate::{
		monitor::{alert, MonitorMetrics},
		moonbeam::MOONBEAM_SCAN_LOG_TARGET,
		Address,
	};

	#[inline]
	fn new_monitor_metrics() -> MonitorMetrics {
		MonitorMetrics {
			target: MOONBEAM_SCAN_LOG_TARGET.to_string(),
			block_number: Some(32.into()),
			error_msg: "Test error message".to_string(),
			keeper_name: "9dD21AdF685CBf76bD3288AEdC5A62b9AddBcd8d".to_owned(),
		}
	}
	#[test]
	fn form_alert_message_should_work() {
		let monitor_metrics = new_monitor_metrics();
		let msg = monitor_metrics.message();
		assert!(msg.is_ok())
	}

	#[tokio::test]
	async fn send_to_bot_should_work() {
		let bot_url = include_str!("../res/bot-url");
		println!("the bot url is {}", bot_url);
		let msg = new_monitor_metrics().message().expect("monitor template format error");
		println!("the messge is \n {:}", &msg);
		let res = alert(bot_url, msg).await;
		assert!(res.is_ok());
	}
}
