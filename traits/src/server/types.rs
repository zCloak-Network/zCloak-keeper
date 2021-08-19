use crate::server::config::ConfigFormat;
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Debug)]
pub struct WebServerState {
	pub base_path: Arc<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct ServerTaskState {
	pub config_path: PathBuf,
	pub config_format: ConfigFormat,
}
