use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use server_traits::server::config::ServerConfig;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TaskManagementConfig {
    pub microkv: MicrokvConfig,
}

impl ServerConfig for TaskManagementConfig {
    fn marker() -> &'static str {
        "task-management"
    }

    fn template() -> Self {
        Self {
            microkv: MicrokvConfig::template(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MicrokvConfig {
    pub base_path: PathBuf,
    pub db_name: Option<String>,
    pub auto_commit: bool,
}

impl ServerConfig for MicrokvConfig {
    fn marker() -> &'static str {
        "task-microkv"
    }

    fn template() -> Self {
        Self {
            base_path: "/tmp/microkv".into(),
            db_name: Some("microkv".to_string()),
            auto_commit: true,
        }
    }
}
