use hyper::{Body, Request, Response};
use microkv::MicroKV;

use crate::utils::server::Resp;
use crate::utils::transfer::{KvListParam, KvOperationParam};
use crate::utils::utils;

fn microkv() -> anyhow::Result<&'static MicroKV> {
    let state = task_management::state::get_state_server_ok()?;
    Ok(state.microkv())
}

pub async fn put(mut req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let param: KvOperationParam = utils::deserialize_body(&mut req).await?;
    let keys = param.keys;
    let values = param.values;
    if keys.len() != values.len() {
        return Resp::<String>::err_with_msg("The length not same by keys and values")
            .response_json();
    }
    if keys.is_empty() {
        return Resp::<String>::err_with_msg("The keys and values is required").response_json();
    }
    let microkv = microkv()?;
    let len = keys.len();
    for i in 0..len {
        let key = keys.get(i).expect("unreachable");
        let value = values.get(i).expect("unreachable");
        microkv.put(key, value)?;
    }
    Resp::<String>::ok().response_json()
}

pub async fn get(mut req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let param: KvOperationParam = utils::deserialize_body(&mut req).await?;
    let keys = param.keys;
    if keys.is_empty() {
        return Resp::<String>::err_with_msg("The keys is required").response_json();
    }
    let microkv = microkv()?;
    let mut values: Vec<Option<String>> = vec![];
    for key in keys {
        let value = microkv.get(key)?;
        values.push(value);
    }
    Resp::ok_with_data(values).response_json()
}

pub async fn list(mut req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let param: KvListParam = utils::deserialize_body(&mut req).await?;
    let sorted = param.sorted;
    let microkv = microkv()?;
    let keys = if sorted {
        microkv.sorted_keys()?
    } else {
        microkv.keys()?
    };
    Resp::ok_with_data(keys).response_json()
}

pub async fn remove(mut req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let param: KvOperationParam = utils::deserialize_body(&mut req).await?;
    let keys = param.keys;
    if keys.is_empty() {
        return Resp::<String>::err_with_msg("The keys is required").response_json();
    }
    let microkv = microkv()?;
    for key in keys {
        microkv.delete(key)?;
    }
    Resp::<String>::ok().response_json()
}
