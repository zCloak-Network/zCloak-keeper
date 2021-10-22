use lifeline::{Bus, Sender};

use server_traits::server::task::{
	ServerSand, ServerTask, ServerTaskKeep, TaskStack, TaskTerminal,
};

use crate::{
	bus::ZcloakTaskBus, config::ZcloakTaskConfig, message::ZcloakTaskMessage,
	service::service::ZcloakNodeService,
};

#[derive(Debug)]
pub struct ZcloakTask {
	stack: TaskStack<ZcloakTaskBus>,
}

impl ServerSand for ZcloakTask {
	const NAME: &'static str = "task-zcloak-substrate";
}

#[async_trait::async_trait]
impl ServerTaskKeep for ZcloakTask {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		self
	}
	async fn route(&self, uri: String, param: serde_json::Value) -> anyhow::Result<TaskTerminal> {
		crate::route::dispatch_route(self.stack.bus(), uri, param).await
	}
}

impl ServerTask<ZcloakTaskBus> for ZcloakTask {
	fn config_template() -> anyhow::Result<serde_json::Value> {
		Ok(serde_json::to_value(ZcloakTaskConfig::template())?)
	}

	fn stack(&mut self) -> &mut TaskStack<ZcloakTaskBus> {
		&mut self.stack
	}
}

impl ZcloakTask {
	pub async fn new(config: ZcloakTaskConfig) -> anyhow::Result<Self> {
		config.store(ZcloakTask::NAME)?;
		let bus = ZcloakTaskBus::default();

		let mut stack = TaskStack::new(bus);
		stack.spawn_service::<ZcloakNodeService>()?;

		let mut sender = stack.bus().tx::<ZcloakTaskMessage>()?;
		sender.send(ZcloakTaskMessage::TaskEvent).await?;
		Ok(Self { stack })
	}
}
