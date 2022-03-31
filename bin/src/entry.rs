use keeper_primitives::{Contract, Http};
use keeper_primitives::{Config, Result, Error, U64, MoonbeamClient, KiltClient, IpfsClient};
use keeper_primitives::config::Error as ConfigError;
use crate::command::StartOptions;
use std::str::FromStr;
use secp256k1::SecretKey;

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

    // run a keeper
    loop {
        run(
            start,
            &moonbeam_client,
            &ipfs_client,
            &kilt_client,
            &proof_contract,
            &aggregator_contract,
            moonbeam_worker_pri
        ).await;
    }

}

// handle detailed process
pub async fn run(
    start: U64,
    moonbeam_client: &MoonbeamClient,
    ipfs_client: &IpfsClient,
    kilt_client: &KiltClient,
    proof_contract: &Contract<Http>,
    aggregator_contract: &Contract<Http>,
    moonbeam_worker_pri: SecretKey
) -> Result<()> {
    let mut start = start;

    loop {
        // 1. scan moonbeam proof event
        let (res, end) = moonbeam::scan_events(start, &moonbeam_client, &proof_contract).await?;
        // reset scan start point
        start = end;
        if res.is_empty() {
            if start == moonbeam_client.best_number().await.map_err(|e| (start, e.into()))? {
                // if current start is the best number, then sleep the block duration.
                use tokio::time::{sleep, Duration};
                sleep(Duration::from_secs(keeper_primitives::moonbeam::MOONBEAM_BLOCK_DURATION)).await;
            }
            continue
        }

        // 2. query ipfs and verify cid proof
        let r = ipfs::query_and_verify(ipfs_client, res).await?;

        // 3. query kilt
        let res = kilt::filter(kilt_client, r).await?;
        // 4. submit tx
        moonbeam::submit_tx(&aggregator_contract, moonbeam_worker_pri, res)
            .await
            .map_err(|e| (start, e.into()))?;

        log::info!("finish batch task");

    }

}


