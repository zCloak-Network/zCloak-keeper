use hyper::{Body, Request};
use server_traits::{error::StandardError, server::task::ServerSand};
use std::path::PathBuf;
use task_zcloak_substrate::task::ZcloakTask;
use task_moonbeam::task::MoonbeamTask;

pub fn base_path(except_path: Option<PathBuf>) -> anyhow::Result<PathBuf> {
	let base_path = except_path.unwrap_or_else(|| {
		let mut path = dirs::home_dir().unwrap_or_else(std::env::temp_dir);
		path.push(".zcloak-server");
		path
	});
	if !base_path.exists() {
		std::fs::create_dir_all(&base_path)?;
	}
	Ok(base_path)
}

pub async fn deserialize_body<T: serde::de::DeserializeOwned>(
	req: &mut Request<Body>,
) -> anyhow::Result<T> {
	let body = req.body_mut();
	match hyper::body::to_bytes(body).await {
		Ok(bytes) => {
			let bytes = bytes.to_vec();
			if bytes.is_empty() {
				return Err(StandardError::Api("The body is required".to_string()).into())
			}
			match serde_json::from_slice::<T>(bytes.as_slice()) {
				Ok(body) => Ok(body),
				Err(e) =>
					return Err(
						StandardError::Api(format!("Failed to deserialize body: {}", e)).into()
					),
			}
		},
		Err(_e) => Err(StandardError::Api("Failed to parse request body".to_string()).into()),
	}
}

pub fn init() -> anyhow::Result<()> {
	init_log();
	init_keep()?;
	Ok(())
}

fn init_log() {
	std::env::set_var(
		"RUST_LOG",
		r#"
        serde=info,
        lifeline=debug,
        zcloak_server=debug,
        support_zcloak_node=debug,
        task_management=debug,
        task_zcloak_substrate=debug,
        task_zcloak_substrate=trace,
		task_moonbeam=debug,
		task_moonbeam=trace,
        primitives=debug,
        components_subxt_client=debug,
        "#,
	);
	std::env::set_var("RUST_BACKTRACE", "1");
	env_logger::init();
}

fn init_keep() -> anyhow::Result<()> {
	task_management::task::add_available_tasks(vec![
		ZcloakTask::NAME,
		MoonbeamTask::NAME,
	])
}
