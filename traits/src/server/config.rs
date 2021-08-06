use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use strum::AsStaticRef;

use crate::error::{VerifyResult, StandardError};

pub trait VerifyConfig {
    fn marker() -> &'static str;
    fn template() -> Self;
}

static INSTANCE: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Debug, Deserialize, Serialize, strum::EnumString, strum::AsStaticStr)]
pub enum ConfigFormat {
    #[strum(serialize = "yml")]
    Yml,
    #[strum(serialize = "json")]
    Json,
    #[strum(serialize = "toml")]
    Toml,
}

impl ConfigFormat {
    pub fn file_extension(&self) -> &'static str {
        self.as_static()
    }
}

pub struct Config;

const DEFAULT_NAMESPACE: &str = "default";

impl Config{
    pub fn default_namespace() -> &'static str {
        DEFAULT_NAMESPACE
    }

    pub fn store<S: AsRef<str>, B: VerifyConfig + Serialize>(
        name: S,
        config: B,
    ) -> VerifyResult<()> {
        Self::store_with_namespace(name, config, DEFAULT_NAMESPACE)
    }

    pub fn store_with_namespace<S: AsRef<str>, B: VerifyConfig + Serialize, N: AsRef<str>>(
        name: S,
        config: B,
        namespace: N,
    ) -> VerifyResult<()> {
        let config_marker = B::marker();
        let key = format!(
            "{}:{}@{}",
            name.as_ref(),
            config_marker,
            namespace.as_ref()
        );

        let json = serde_json::to_string(&config).map_err(|e| {
            StandardError::Other(format!(
                "Te config cannot be serialize, lease check it. [{}] {:?}",
                key, e
            ))
        })?;
        let _mutex = INSTANCE.lock().unwrap().insert(key, json);
        Ok(())
    }


}