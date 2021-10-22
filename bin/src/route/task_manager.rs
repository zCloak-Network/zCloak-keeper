use std::{ffi::OsStr, path::PathBuf, str::FromStr};

use server_traits::{
	error::StandardError,
	server::{
		config::{Config, ConfigFormat},
		task::{ServerSand, ServerTask},
		types::ServerTaskState,
	},
};
use task_zcloak_substrate::task::ZcloakTask;
use task_moonbeam::task::MoonbeamTask;

use crate::utils::transfer::{TaskConfigTemplateParam, TaskStartParam};

/// Auto start all configured task
pub async fn auto_start_task(base_path: PathBuf) -> anyhow::Result<()> {
	let available_tasks = task_management::task::available_tasks()?;
	let read_dir: Vec<PathBuf> = std::fs::read_dir(&base_path)?
		.into_iter()
		.filter(|r| r.is_ok())
		.map(|r| r.unwrap().path())
		.filter(|r| r.is_file())
		.collect();
	let all_tasks = available_tasks.iter().collect::<Vec<&String>>();

	for task in all_tasks {
		if let Some(task_config) = read_dir
			.iter()
			.find(|path| path.file_name().and_then(OsStr::to_str).unwrap_or("").starts_with(task))
		{
			let format = task_config.extension().and_then(OsStr::to_str).ok_or_else(|| {
				StandardError::Api(format!("Failed to extra config format for [{}]", task))
			})?;
			let param = TaskStartParam {
				format: ConfigFormat::from_str(format).map_err(|_e| {
					StandardError::Api(format!("Failed to extra config format for [{}]", task))
				})?,
				name: task.clone(),
				config: None,
				password: None,
				store_password: false,
			};
			start_task_single(base_path.clone(), param).await?;
		}
	}

	Ok(())
}

/// Start a single task
pub async fn start_task_single(base_path: PathBuf, param: TaskStartParam) -> anyhow::Result<()> {
	let name = &param.name[..];
	if task_management::task::task_is_running(name) {
		return Err(StandardError::Api(format!("The task [{}] is running", &param.name)).into())
	}

	let config_format = param.format;
	let option_config = &param.config;

	if !task_management::task::is_available_task(name) {
		return Err(StandardError::Api(format!("Not support this task [{}]", &param.name)).into())
	}
	let path_config = base_path.join(format!("{}.{}", name, config_format.file_extension()));
	if let Some(config_raw) = option_config {
		Config::persist_raw(&path_config, &config_raw)?;
	}
	if !path_config.exists() {
		return Err(
			StandardError::Api(format!("The config file not found: {:?}", path_config)).into()
		)
	}

	let state_server = task_management::state::get_state_server_ok()?;

	// put task password
	if let Some(password) = param.password {
		state_server.put_task_config_password(name, password, param.store_password)?;
	}
	match name {
		ZcloakTask::NAME => {
			let task_config = Config::load(&path_config)?;
			let task = ZcloakTask::new(task_config).await?;
			task_management::task::keep_task(ZcloakTask::NAME, Box::new(task))?;
		}
		MoonbeamTask::NAME => {
			let task_config = Config::load(&path_config)?;
			let task = MoonbeamTask::new(task_config).await?;
			task_management::task::keep_task(MoonbeamTask::NAME, Box::new(task))?;
		}

		_ => return Err(StandardError::Api(format!("Unsupported task: [{}]", name)).into()),
	};

	// keep task state
	let state_task =
		ServerTaskState { config_path: path_config.clone(), config_format: config_format.clone() };
	task_management::state::set_state_task(name, state_task)?;

	Ok(())
}

/// Generate task config template
pub fn task_config_template(param: TaskConfigTemplateParam) -> anyhow::Result<String> {
	let task_name = param.name;
	let format = param.format;
	if !task_management::task::is_available_task(&task_name) {
		return Err(StandardError::Api(format!("Not support this task [{}]", &task_name)).into())
	}
	let value = match &task_name[..] {
		ZcloakTask::NAME => ZcloakTask::config_template(),
		_ =>
			return Err(StandardError::Api(format!(
				"Unsupported to show default config template: [{}]",
				task_name
			))
			.into()),
	}?;
	let template = Config::raw_config(value, format)?;
	Ok(template)
}
