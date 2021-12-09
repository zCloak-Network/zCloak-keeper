use lifeline::{Bus, Sender};

use server_traits::server::task::{
	ServerSand, ServerTask, ServerTaskKeep, TaskStack, TaskTerminal,
};

use crate::{
	bus::MoonbeamTaskBus, config::MoonbeamTaskConfig, message::MoonbeamTaskMessage,
	service::moonbeam::MoonBeamService,
};

#[derive(Debug)]
pub struct MoonbeamTask {
	stack: TaskStack<MoonbeamTaskBus>,
}

impl ServerSand for MoonbeamTask {
	const NAME: &'static str = "task-moonbeam";
}

#[async_trait::async_trait]
impl ServerTaskKeep for MoonbeamTask {
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

impl ServerTask<MoonbeamTaskBus> for MoonbeamTask {
	fn config_template() -> anyhow::Result<serde_json::Value> {
		Ok(serde_json::to_value(MoonbeamTaskConfig::template())?)
	}

	fn stack(&mut self) -> &mut TaskStack<MoonbeamTaskBus> {
		&mut self.stack
	}
}

impl MoonbeamTask {
	pub async fn new(config: MoonbeamTaskConfig) -> anyhow::Result<Self> {
		config.store(MoonbeamTask::NAME)?;
		let bus = MoonbeamTaskBus::default();

		let mut stack = TaskStack::new(bus);
		stack.spawn_service::<MoonBeamService>()?;

		let mut sender = stack.bus().tx::<MoonbeamTaskMessage>()?;
		sender.send(MoonbeamTaskMessage::ListenMoonbeam).await?;
		Ok(Self { stack })
	}
}
