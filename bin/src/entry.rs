use std::{str::FromStr, sync::Arc, time::Duration};
use log::log;
use secp256k1::SecretKey;
use tokio::{io, sync::RwLock};
use yaque::{channel, recovery};

// #[cfg(feature = "monitor")]
use keeper_primitives::monitor;

use keeper_primitives::{
	config::Error as ConfigError, Config, ConfigInstance, Contract, Error, EventResult,
	Http, IpfsClient, JsonParse, KiltClient, MoonbeamClient, Result, VerifyResult, U64,
};

use crate::command::StartOptions;

pub async fn start(start_options: StartOptions) -> std::result::Result<(), Error> {
	// load config
	let start: U64 = start_options.start_number.unwrap_or_default().into();
	let channel_files = start_options.channel_files()?;
	let config_path = start_options.config.ok_or::<Error>(
		ConfigError::OtherError("Config File need to be specific".to_owned()).into(),
	)?;
	let config = Config::load_from_json(&config_path)?;


	log::info!("[Config] load successfully!");
	// init config，
	let moonbeam_client = MoonbeamClient::new(config.moonbeam.url)?;
	let ipfs_client = IpfsClient::new(&config.ipfs.base_url)?;
	let kilt_client = KiltClient::try_from_url(&config.kilt.url).await?;

	let proof_contract = moonbeam_client.proof_contract(&config.moonbeam.read_contract)?;
	let aggregator_contract =
		moonbeam_client.aggregator_contract(&config.moonbeam.write_contract)?;

	let moonbeam_worker_pri = secp256k1::SecretKey::from_str(&config.moonbeam.private_key)?;

	let config_instance = ConfigInstance {
		channel_files,
		moonbeam_client,
		ipfs_client,
		kilt_client,
		proof_contract,
		aggregator_contract,
		private_key: moonbeam_worker_pri,
	};

	// run a keeper
	run(start, Arc::new(RwLock::new(config_instance))).await?;

	Ok(())
}

// handle detailed process
pub async fn run(
	start: U64,
	configs: Arc<RwLock<ConfigInstance>>,
) -> std::result::Result<(), keeper_primitives::Error> {
	// it record the latest block that contains proofevents
	// used in ganache
	let mut start = start;

	// get channel files
	let config = configs.clone();
	let config_channels = &config.read().await.channel_files;

	// force recover all channels, which delete all '.lock' files
	recovery::unlock_queue(&config_channels.event_to_ipfs).expect("fail to unlock event2ipfs channel");
	recovery::unlock_queue(&config_channels.verify_to_attest)
		.expect("fail to unlock verify2attestation channel");
	recovery::unlock_queue(&config_channels.attest_to_submit)
		.expect("fail to unlock attestation2submit channel");


	let (mut event_sender, mut event_receiver) = channel(&config_channels.event_to_ipfs).unwrap();
	let (mut attest_sender, mut attest_receiver) = channel(&config_channels.verify_to_attest).unwrap();
	let (mut submit_sender, mut submit_receiver) = channel(&config_channels.attest_to_submit).unwrap();

	// alert message sending
	let (monitor_sender, mut monitor_receiver) = tokio::sync::mpsc::channel::<monitor::MonitorMessage>(100);

	// spread configs
	let config1 = configs.clone();
	let config2 = configs.clone();
	let config3 = configs.clone();
	let config4 = configs.clone();

	let monitor_sender1 = monitor_sender.clone();
	let monitor_sender2 = monitor_sender.clone();
	let monitor_sender3 = monitor_sender.clone();
	let monitor_sender4 = monitor_sender.clone();


	// 1. scan moonbeam proof event, and push them to event channel
	let task_scan = tokio::spawn(async move {
		let config = config1.read().await;
		moonbeam::task_scan(&config, &mut event_sender, start, monitor_sender1).await;
	});

	// 2. query ipfs and verify cid proof
	// TODO: separate ipfs query end starksvm verify
	let task_ipfs_verify = tokio::spawn(async move {
		let config = config2.read().await;
		ipfs::task_verify(&config, (&mut attest_sender, &mut event_receiver), monitor_sender2).await;
	});

	//
	// 3. query kilt
	let task_kilt_attest = tokio::spawn(async move {
		let config = config3.read().await;
		kilt::task_attestation(&config, (&mut submit_sender, &mut attest_receiver), monitor_sender3).await;

	});

	// 4. submit tx
	let task_submit_tx = tokio::spawn(async move {
		let config = config4.read().await;
		moonbeam::task_submit(&config, &mut submit_receiver, monitor_sender4).await;
	});

	// monitor
	let task_monitor_handle = tokio::spawn(async move {
		while let Some(msg) = monitor_receiver.recv().await {
			if cfg!(feature = "monitor") {
				//todo:  send msg to robot
			}
		}
	});

	tokio::try_join!(task_scan, task_ipfs_verify, task_kilt_attest, task_submit_tx, task_monitor_handle)?;
	Ok(())
}
