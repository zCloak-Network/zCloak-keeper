use std::os::macos::raw::stat;
use std::str::FromStr;
use std::sync::Arc;

use secp256k1::SecretKey;
use tokio::io;
use tokio::sync::RwLock;
use yaque::channel;

use keeper_primitives::{Contract, Http, U64, VerifyResult};
use keeper_primitives::{
    Config, Error, EventResult,
    IpfsClient, JsonParse, KiltClient, MoonbeamClient, Result
};
use keeper_primitives::config::Error as ConfigError;

use crate::command::StartOptions;

#[derive(Clone)]
pub struct ConfigInstance {
    pub(crate) moonbeam_client: MoonbeamClient,
    pub(crate) ipfs_client: IpfsClient,
    pub(crate) kilt_client: KiltClient,
    pub(crate) proof_contract: Contract<Http>,
    pub(crate) aggregator_contract: Contract<Http>,
    pub(crate) private_key: SecretKey,
}


const CHANNEL_LOG_TARGET: &str = "CHANNEL";

pub async fn start(
    start_options: StartOptions,
) -> std::result::Result<(), Error> {
    // load config
    let start: U64 = start_options.start_number.unwrap_or_default().into();
    let config_path = start_options.config.ok_or::<Error>(ConfigError::OtherError("Config File need to be specific".to_owned()).into())?;
    let config = Config::load_from_json(&config_path)?;

    // init config
    let moonbeam_client = MoonbeamClient::new(config.moonbeam.url)?;
    let ipfs_client = IpfsClient::new(&config.ipfs.base_url)?;
    let kilt_client = KiltClient::try_from_url(&config.kilt.url).await?;

    let proof_contract = moonbeam_client.proof_contract(&config.moonbeam.read_contract)?;
    let aggregator_contract = moonbeam_client.aggregator_contract(&config.moonbeam.write_contract)?;

    let moonbeam_worker_pri = secp256k1::SecretKey::from_str(&config.moonbeam.private_key)?;

    let config_instance = ConfigInstance {
        moonbeam_client: moonbeam_client,
        ipfs_client: ipfs_client,
        kilt_client: kilt_client,
        proof_contract,
        aggregator_contract,
        private_key: moonbeam_worker_pri,
    };

    // run a keeper
    run(
        start,
        Arc::new(RwLock::new(config_instance)),
    ).await;


    Ok(())
}

// handle detailed process
pub async fn run(
    start: U64,
    configs: Arc<RwLock<ConfigInstance>>,
) -> Result<()> {
    let mut start = start;
    let (mut event_sender, mut event_receiver) = channel("../data/event").unwrap();
    let (mut attest_sender, mut attest_receiver) = channel("../data/attestation").unwrap();
    let (mut submit_sender, mut submit_receiver) = channel("../data/submit").unwrap();

    let config1 = configs.clone();
    let config2 = configs.clone();
    let config3 = configs.clone();
    let config4 = configs.clone();

    // 1. scan moonbeam proof event, and push them to event channel
    tokio::spawn(async move {
        // TODO: handle unwrap
        let config = config1.read().await;
        loop {
            let res;
            let end;
            match moonbeam::scan_events(start, &config.moonbeam_client, &config.proof_contract).await {
                Ok(r) => {
                    res = r.0;
                    end = r.1
                },
                Err(e) => {
                    // repeat scanning from the start again
                    start = e.0;
                    continue;
                }
            }

            if !res.is_empty() {
                // send result to channel
                // TODO: handle error
                let output = res.into_json_str().unwrap();
                let status = event_sender.send(output).await;
                if let Err(_) = status {
                    log::error!(
                        target: CHANNEL_LOG_TARGET,
                        "Fail to write data in block from: #{:?} into event channel file",
                        start,
                    );
                    // repeat scanning from the start again
                    continue;
                }
            } else {
                let latest = &config.moonbeam_client.best_number().await.unwrap_or_default();
                if start == *latest {
                    // if current start is the best number, then sleep the block duration.
                    use tokio::time::{sleep, Duration};
                    sleep(Duration::from_secs(keeper_primitives::moonbeam::MOONBEAM_BLOCK_DURATION)).await;
                }
                // continue
            }

            // reset scan start point
            start = end;
        }
    });


    // 2. query ipfs and verify cid proof
    // TODO: seperate ipfs query end starksvm verify
    tokio::spawn(async move {
        let config = config2.read().await;
        // TODO: handle unwrap
        while let Ok(events) = event_receiver.recv().await {
            // parse event from str to ProofEvent
            // TODO: handle error
            let inputs = EventResult::from_json_str(&*events);
            let r = ipfs::query_and_verify(&config.ipfs_client, inputs).await;
            let res = match r {
                Ok(v) => v,
                Err(e) => continue,
            };
            // TODO; handle unwrap
            let status = attest_sender.send(serde_json::to_vec(&res).unwrap()).await;

            match status {
                Ok(_) => {
                    // delete events in channel after the events are successfully
                    // transformed and pushed into
                    events.commit();
                },
                Err(e) => continue,
            }
        }
    });

    //
    // 3. query kilt
    tokio::spawn(async move {
        let config = config3.read().await;
        // TODO: handle unwrap
        while let Ok(r) = attest_receiver.recv().await {
            // parse verify result from str to VerifyResult
            // TODO: handle error
            let inputs = serde_json::from_slice(&*r).unwrap();
            let res = kilt::filter(&config.kilt_client, inputs).await;
            let verify_res = match res {
                Ok(r) => r,
                Err(_) => continue,
            };

            // TODO: handle unwrap
            let status = submit_sender.send(serde_json::to_vec(&verify_res).unwrap()).await;

            match status {
                Ok(_) => {
                    r.commit();
                },
                Err(e) => continue,
            }
        }
    });


    // 4. submit tx
    tokio::spawn(async move {
        let config = config4.read().await;
        while let Ok(r) = submit_receiver.recv().await {
            // TODO: handle unwrap
            let inputs = serde_json::from_slice(&*r).unwrap();
            let res = moonbeam::submit_tx(&config.aggregator_contract, config.private_key, inputs)
                .await;
            ;
            match res {
                Ok(_) => {
                    r.commit();
                },
                Err(e) => continue,
            };
        }
    });

    Ok(())
}


