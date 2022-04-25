use std::{str::FromStr, sync::Arc, time::Duration};

use keeper_primitives::{
	config::Error as ConfigError, kilt::KILT_LOG_TARGET, Config, ConfigInstance, Contract, Error,
	Http, IpfsClient, JsonParse, Key, KiltClient, MoonbeamClient, Result,
	SecretKeyRef, VerifyResult, U64,
};
use log::log;
use secp256k1::SecretKey;
use tokio::{io, sync::RwLock};
use yaque::{channel, recovery};
// #[cfg(feature = "monitor")]
use keeper_primitives::{monitor, monitor::MonitorMetrics};
use keeper_primitives::ipfs::IPFS_LOG_TARGET;
use keeper_primitives::moonbeam::{MOONBEAM_SCAN_LOG_TARGET, MOONBEAM_SUBMIT_LOG_TARGET};

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
	let key_ref = SecretKeyRef::new(&moonbeam_worker_pri);
	let keeper_address = key_ref.address();

	#[cfg(feature = "monitor")]
	let bot_url = config.monitor.bot_url;

	let config_instance = ConfigInstance {
		channel_files,
		moonbeam_client,
		ipfs_client,
		kilt_client,
		proof_contract,
		aggregator_contract,
		private_key: moonbeam_worker_pri,
		keeper_address,
		#[cfg(feature = "monitor")]
		bot_url,
	};

	log::info!("ConfigInstance initialized");

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
	recovery::unlock_queue(&config_channels.event_to_ipfs)
		.expect("fail to unlock event2ipfs channel");
	recovery::unlock_queue(&config_channels.verify_to_attest)
		.expect("fail to unlock verify2attestation channel");
	recovery::unlock_queue(&config_channels.attest_to_submit)
		.expect("fail to unlock attestation2submit channel");

	let (mut event_sender, mut event_receiver) = channel(&config_channels.event_to_ipfs).unwrap();
	let (mut attest_sender, mut attest_receiver) =
		channel(&config_channels.verify_to_attest).unwrap();
	let (mut submit_sender, mut submit_receiver) =
		channel(&config_channels.attest_to_submit).unwrap();

	// alert message sending
	let (monitor_sender, mut monitor_receiver) =
		tokio::sync::mpsc::channel::<monitor::MonitorMetrics>(100);

	// spread configs
	let config1 = configs.clone();
	let config2 = configs.clone();
	let config3 = configs.clone();
	let config4 = configs.clone();
	let config5 = configs.clone();

	// spread monitors
	let monitor_sender1 = monitor_sender.clone();
	let monitor_sender2 = monitor_sender.clone();
	let monitor_sender3 = monitor_sender.clone();
	let monitor_sender4 = monitor_sender.clone();

	// 1. scan moonbeam proof event, and push them to event channel
	let task_scan = tokio::spawn(async move {
		log::info!("Start Task Scan");
		let config = config1.read().await;
		let res =
			moonbeam::task_scan(&config, &mut event_sender, start, monitor_sender1.clone()).await;
		if let Err(e) = res {
			if cfg!(feature = "monitor") {
				let monitor_metrics = MonitorMetrics::new(
					MOONBEAM_SCAN_LOG_TARGET.to_string(),
					e.0,
					e.1.into(),
					config.keeper_address,
				);
				monitor_sender1.send(monitor_metrics).await;
			}
		}
	});

	// 2. query ipfs and verify cid proof
	// TODO: separate ipfs query end starksvm verify
	let task_ipfs_verify = tokio::spawn(async move {
		let config = config2.read().await;
		let res = ipfs::task_verify(&config, (&mut attest_sender, &mut event_receiver)).await;

		if let Err(e) = res {
			log::error!(
				//todo: config
				target: "IPFS_AND_VERIFY",
				"encounter error: {:?}",
				e
			);
			if cfg!(feature = "monitor") {
				let monitor_metrics = MonitorMetrics::new(
					IPFS_LOG_TARGET.to_string(),
					e.0,
					e.1.into(),
					config.keeper_address,
				);
				monitor_sender2.send(monitor_metrics).await;
			}
		}
	});

	//
	// 3. query kilt
	let task_kilt_attest = tokio::spawn(async move {
		let config = config3.read().await;
		let res = kilt::task_attestation(&config, (&mut submit_sender, &mut attest_receiver)).await;

		if let Err(e) = res {
			log::error!(target: KILT_LOG_TARGET, "encounter error: {:?}", e);

			if cfg!(feature = "monitor") {
				let monitor_metrics = MonitorMetrics::new(
					KILT_LOG_TARGET.to_string(),
					e.0,
					e.1.into(),
					config.keeper_address,
				);
				monitor_sender3.send(monitor_metrics).await;
			}
		}
	});

	// 4. submit tx
	let task_submit_tx = tokio::spawn(async move {
		let config = config4.read().await;
		let res =
			moonbeam::task_submit(&config, &mut submit_receiver, monitor_sender4.clone()).await;

		if cfg!(feature = "monitor") {
			if let Err(e) = res {
				let monitor_metrics = MonitorMetrics::new(
					MOONBEAM_SUBMIT_LOG_TARGET.to_string(),
					e.0,
					e.1,
					config.keeper_address,
				);
				monitor_sender4.send(monitor_metrics).await;
			}
		}
	});

	// monitor
	let task_monitor_handle = tokio::spawn(async move {
		let config = config5.read().await;
		while let Some(msg) = monitor_receiver.recv().await {
			#[cfg(feature = "monitor")]
			{
				let bot_url = &config.bot_url;
				monitor::alert(&bot_url, msg.message().expect("monitor message parse wrong")).await;
			}
			// else do nothing
		}
	});

	// all tasks will loop so no need to handle Ok condition
	tokio::try_join!(
		task_scan,
		task_ipfs_verify,
		task_kilt_attest,
		task_submit_tx,
		task_monitor_handle
	)?;
	Ok(())
}
