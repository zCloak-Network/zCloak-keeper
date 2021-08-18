use server_traits::server::task::TaskTerminal;

use crate::bus::ZcloakTaskBus;

pub async fn dispatch_route(
    _bus: &ZcloakTaskBus,
    uri: String,
    param: serde_json::Value,
) -> anyhow::Result<TaskTerminal> {
    let value = TaskTerminal::new(format!("{} -> {:?}", uri, param));
    Ok(value)
}
