use server_traits::server::task::TaskTerminal;

use crate::bus::MoonbeamTaskBus;

pub async fn dispatch_route(
	_bus: &MoonbeamTaskBus,
	uri: String,
	param: serde_json::Value,
) -> anyhow::Result<TaskTerminal> {
	let value = TaskTerminal::new(format!("{} -> {:?}", uri, param));
	Ok(value)
}
