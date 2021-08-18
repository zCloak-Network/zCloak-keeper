use once_cell::sync::{Lazy, OnceCell};
use server_traits::error::StandardError;
use server_traits::server::types::{ServerTaskState, WebServerState};
use crate::resource::TaskResource;
use std::collections::HashMap;
use std::sync::Mutex;
 

static STATE_SERVER: OnceCell<TaskResource> = OnceCell::new();

pub fn set_state_server(state: TaskResource) -> anyhow::Result<()> {
    STATE_SERVER.set(state).map_err(|_e| StandardError::Api("Failed to set server state".to_string()).into())
}

pub fn get_state_server() -> Option<&'static TaskResource> {
    STATE_SERVER.get()
}

pub fn get_state_server_ok() -> anyhow::Result<&'static TaskResource> {
    get_state_server().ok_or_else(|| StandardError::Api("Please set server state first.".to_string()).into())
}

static STATE_WEBSITE: OnceCell<WebServerState> = OnceCell::new();

pub fn set_state_website(state: WebServerState) -> anyhow::Result<()> {
    STATE_WEBSITE
        .set(state)
        .map_err(|_e| StandardError::Api("Failed to keep website state".to_string()).into())
}

pub fn get_state_website() -> Option<WebServerState> {
    STATE_WEBSITE.get().cloned()
}

pub fn get_state_website_unwrap() -> WebServerState {
    get_state_website()
        .ok_or_else(|| StandardError::Api("Please set website state first.".to_string()))
        .unwrap()
}


static STATE_TASK: Lazy<Mutex<HashMap<String, ServerTaskState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn set_state_task(task: impl AsRef<str>, state: ServerTaskState) -> anyhow::Result<()> {
    let mut state_task = STATE_TASK
        .lock()
        .map_err(|_e| StandardError::Api("failed to get task state".to_string()))?;
    state_task.insert(task.as_ref().to_string(), state);
    Ok(())
}

pub fn get_state_task(task: impl AsRef<str>) -> anyhow::Result<Option<ServerTaskState>> {
    let state_task = STATE_TASK
        .lock()
        .map_err(|_e| StandardError::Api("failed to get task state".to_string()))?;
    Ok(state_task.get(task.as_ref()).cloned())
}

pub fn get_state_task_unwrap(task: impl AsRef<str>) -> anyhow::Result<ServerTaskState> {
    match get_state_task(task) {
        Ok(v) => match v {
            Some(t) => Ok(t),
            None => Err(StandardError::Api("failed to get task state".to_string()).into()),
        },
        Err(e) => Err(e),
    }
}


