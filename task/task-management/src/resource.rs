use std::fmt::{Debug, Formatter};

use microkv::MicroKV;

use server_traits::{
	error::ServerResult,
	server::{component::ServerComponent, config::ServerConfig, task::ServerSand},
};

use crate::config::{MicrokvConfig, TaskManagementConfig};

#[derive(Clone)]
pub struct TaskManagementComponent {
	config: TaskManagementConfig,
}

impl TaskManagementComponent {
	pub fn new(config: TaskManagementConfig) -> Self {
		Self { config }
	}
}

#[async_trait::async_trait]
impl ServerComponent<TaskManagementConfig, TaskResource> for TaskManagementComponent {
	fn restore_with_namespace<T: ServerSand>(_namespace: String) -> ServerResult<Self> {
		panic!("PANIC: THE ZCLOAK SERVER STATE CAN NOT RESTORE FROM CONFIG, PLEASE INIT IT FROM PROGRAM ENTRYPOINT AND SHARE IT")
	}

	async fn component(&self) -> anyhow::Result<TaskResource> {
		let config_microkv = &self.config.microkv;
		let dbname = config_microkv
			.db_name
			.clone()
			.unwrap_or_else(|| MicrokvConfig::marker().to_string());
		let kv = MicroKV::open_with_base_path(dbname, config_microkv.base_path.clone())?
			.set_auto_commit(config_microkv.auto_commit);
		Ok(TaskResource { microkv: kv })
	}

	fn config(&self) -> &TaskManagementConfig {
		&self.config
	}
}

#[derive(Clone)]
pub struct TaskResource {
	microkv: MicroKV,
}

lifeline::impl_storage_clone!(TaskResource);

impl TaskResource {
	pub fn microkv(&self) -> &MicroKV {
		&self.microkv
	}
	pub fn put_task_config_password(
		&self,
		task: impl AsRef<str>,
		password: impl AsRef<str>,
		store: bool,
	) -> anyhow::Result<()> {
		let task = task.as_ref();
		let password = password.as_ref();
		crate::keep::put_task_config_password(task, password)?;
		if store {
			let key = format!("{}@password", task);
			self.microkv().put(key, &password.to_string())?;
		}
		Ok(())
	}
	pub fn get_task_config_password(
		&self,
		task: impl AsRef<str>,
	) -> anyhow::Result<Option<String>> {
		let task = task.as_ref();
		let key = format!("{}@password", task);
		match self.microkv().get(key)? {
			Some(v) => Ok(Some(v)),
			None => crate::keep::get_task_config_password(task),
		}
	}
	pub fn get_task_config_password_unwrap_or_default(
		&self,
		task: impl AsRef<str>,
	) -> anyhow::Result<String> {
		Ok(self.get_task_config_password(task)?.unwrap_or_default())
	}
}

impl Debug for TaskResource {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str("TaskResource { microkv: <...> }")
	}
}
