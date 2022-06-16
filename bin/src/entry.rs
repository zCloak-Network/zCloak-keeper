use log::info;
use std::{str::FromStr};
use crate::error::Error;
use yaque::{channel, recovery};
use ipfs::{IpfsClient, IpfsService};
use moonbeam::{MOONBEAM_SCAN_LOG_TARGET, MOONBEAM_SUBMIT_LOG_TARGET, MOONBEAM_QUERY_LOG_TARGET};
use moonbeam::Error as MoonbeamError;
use kilt::Error as KiltError;
use ipfs::Error as IpfsError;
use kilt::KILT_LOG_TARGET;
use ipfs::IPFS_LOG_TARGET;

use keeper_primitives::{
	Key,
	monitor::{self, NotifyingMessage}, SecretKeyRef, U64,
};
use keeper_primitives::keeper::KeeperSetting;
use kilt::{KiltClient, KiltService};
use moonbeam::{MoonbeamService, MoonbeamServiceBuilder};
use crate::{command::StartOptions, metrics::TOKIO_THREADS_TOTAL};

use prometheus_endpoint::{init_prometheus, PrometheusConfig, Registry};
use crate::config::{ChannelFiles, Config, Error as ConfigError};
use crate::metrics::register_globals;

const SLEEP_SECS: u64 = 1;

// TODO move
async fn sleep() {
	info!("sleep for web3 error, waiting for {:} secs", SLEEP_SECS);
	tokio::time::sleep(std::time::Duration::from_secs(SLEEP_SECS)).await;
}

pub async fn start(start_options: StartOptions) -> std::result::Result<(), Error> {
	// load config
	let start: U64 = start_options.start_number.unwrap_or_default().into();
	let channel_files = start_options.channel_files()?;
	let config_path = start_options.config.ok_or::<Error>(
		ConfigError::OtherError("Config File need to be specific".to_owned()).into(),
	)?;
	let config = Config::load_from_json(&config_path)?;

	log::info!("[Config] load successfully!");

	let moonbeam_worker_pri = secp256k1::SecretKey::from_str(&config.moonbeam.private_key)?;
	let key_ref = SecretKeyRef::new(&moonbeam_worker_pri);
	let keeper_address = key_ref.address();

	let keeper_setting = KeeperSetting::new(keeper_address).await;

	#[cfg(feature = "monitor")]
	let bot_url = config.notify_bot.bot_url;

	// init prometheus registry
	let prometheus_registry = if let Some(port) = start_options.prometheus_port {
		let prometheus_config =
			PrometheusConfig::new_with_default_registry(port, hex::encode(&keeper_address));
		let registry = prometheus_config.prometheus_registry();
		register_globals(&(registry))?;
		let registry1 = registry.clone();
		// init prometheus client
		tokio::spawn(async move {
			init_prometheus(port, registry1).await;
			log::info!("Prometheus client is on.");
		});

		Some(registry)
	} else {
		None
	};

	// build moonbeam service
	let moonbeam_service_builder = MoonbeamServiceBuilder::new(config.moonbeam);
	// todo: remember to inject metrics and registry
	let moonbeam_service = moonbeam_service_builder
		.inject_keeper_setting(keeper_setting.clone())
		.build()
		// todo: throw up
		.expect("moonbeam service construct wrong.");

	// build ipfs service
	// todo: add metric
	let ipfs_service = IpfsService::new(&config.ipfs.base_url);

	// build kilt service
	let kilt_service = KiltService::new(&config.kilt.url).await;

	log::info!("ConfigInstance initialized");

	// run a keeper
	run(start, moonbeam_service, ipfs_service, kilt_service, channel_files, keeper_setting).await?;

	Ok(())
}

// handle detailed process
// todo: extract same logic to a function
// todo: handle monitor sender error
pub async fn run(
	start: U64,
	moonbeam_service: MoonbeamService,
	ipfs_service: IpfsService,
	kilt_service: KiltService,
	channel_files: ChannelFiles,
	// todo:remove
	keeper_setting: KeeperSetting
) -> std::result::Result<(), Error> {
	// it record the latest block that contains proofevents
	// used in ganache
	let start = start;

	// force recover all channels, which delete all '.lock' files
	recovery::unlock_queue(&channel_files.event_to_ipfs)
		.expect("fail to unlock event2ipfs channel");
	recovery::unlock_queue(&channel_files.verify_to_attest)
		.expect("fail to unlock verify2attestation channel");
	recovery::unlock_queue(&channel_files.attest_to_submit)
		.expect("fail to unlock attestation2submit channel");

	let (mut event_sender, mut event_receiver) = channel(&channel_files.event_to_ipfs).unwrap();
	let (mut attest_sender, mut attest_receiver) =
		channel(&channel_files.verify_to_attest).unwrap();
	let (mut submit_sender, mut submit_receiver) =
		channel(&channel_files.attest_to_submit).unwrap();

	// alert message sending
	let (monitor_sender, mut monitor_receiver) =
		tokio::sync::mpsc::channel::<NotifyingMessage>(100);

	// spread monitors
	let monitor_sender1 = monitor_sender.clone();
	let monitor_sender2 = monitor_sender.clone();
	let monitor_sender3 = monitor_sender.clone();
	let monitor_sender4 = monitor_sender.clone();

	// register global metrics to prometheus
	// let unwrap_config = configs.read().await;
	// let registry = unwrap_config.clone().prometheus_registry;
	// if registry.is_some() {
	// 	// todo : ugly hacking
	// 	super::metrics::register_globals(&(registry.unwrap()))?;
	// }


	// 1. scan moonbeam proof event, and push them to event channel
	let moonbeam_service_for_scan = moonbeam_service.clone();
	let task_scan = tokio::spawn(async move {
		TOKIO_THREADS_TOTAL.inc();
		let mut count = 0;
		loop {
			log::info!("Start Task Scan...[{}]", count);
			count += 1;
			let res =
				moonbeam::task_scan(&moonbeam_service_for_scan, &mut event_sender, start, monitor_sender1.clone())
					.await.map_err(|e| (e.0, e.1.into()));
			// handle error
			if let Err(e) = res {
				if cfg!(feature = "monitor") {
				// 	let monitor_metrics = NotifyingMessage::new(
				// 		MOONBEAM_SCAN_LOG_TARGET.to_string(),
				// 		e.0,
				// 		&e.1,
				// 		config.keeper_address,
				// 		&config.moonbeam_client.ip_address,
				// 	);
				// 	monitor_sender1.send(monitor_metrics).await;
				}

				match e.1 {
					// connection error, do nothing, just re scan
					Error::MoonbeamError(MoonbeamError::Web3Error(_)) |
					Error::MoonbeamError(MoonbeamError::Web3ContractError(_)) |
					Error::TimeOutError(_) => {
						// todo: make this more tolerant, e.g. retry N times first before throw and
						// quit
						sleep().await;
						continue
					},
					_ => return e,
				};
			}
		}
	});

	// 2. query ipfs and verify cid proof
	// TODO: separate ipfs query end starksvm verify
	let task_ipfs_verify = tokio::spawn(async move {
		TOKIO_THREADS_TOTAL.inc();
		loop {
			let res = ipfs::task_verify(&ipfs_service, (&mut attest_sender, &mut event_receiver)).await.map_err(|e| (e.0, e.1.into()));;
			if let Err(e) = res {
				log::error!(
					//todo: config
					target: "IPFS_AND_VERIFY",
					"encounter error: {:?} in block: {:?}",
					e.1,
					e.0
				);

				if cfg!(feature = "monitor") {
					// let monitor_metrics = NotifyingMessage::new(
					// 	IPFS_LOG_TARGET.to_string(),
					// 	e.0,
					// 	&e.1,
					// 	config.keeper_address,
					// 	&config.ipfs_client.ip_address,
					// );
					// monitor_sender2.send(monitor_metrics).await;
				}
				// start refetching ipfs proof if connection error encountered
				match e.1 {
					Error::IpfsError(ipfs::Error::HttpError(_)) => {
						// TODO move retry here
						sleep().await;
						continue
					},
					_ => return e,
				};
			}
		}
	});

	//
	// 3. query kilt
	let task_kilt_attest = tokio::spawn(async move {
		TOKIO_THREADS_TOTAL.inc();
		loop {
			let res =
				kilt::task_attestation(&kilt_service, (&mut submit_sender, &mut attest_receiver)).await.map_err(|e| (e.0, e.1.into()));;

			if let Err(e) = res {
				log::error!(target: KILT_LOG_TARGET, "encounter error: {:?}", e);
				if cfg!(feature = "monitor") {
					// let monitor_metrics = NotifyingMessage::new(
					// 	KILT_LOG_TARGET.to_string(),
					// 	e.0,
					// 	&e.1,
					// 	config.keeper_address,
					// 	&config.kilt_client.ip_address,
					// );
					// monitor_sender3.send(monitor_metrics).await;
				}

				match e.1 {
					Error::KiltError(KiltError::KiltClientError(_)) | Error::TimeOutError(_) => {
						// TODO need retry
						sleep().await;
						continue
					},
					_ => return e,
				};
			}
		}
	});

	// 4. submit tx
	let task_submit_tx = tokio::spawn(async move {
		TOKIO_THREADS_TOTAL.inc();

		// todo: register tx submitted metrics

		loop {
			let res = moonbeam::task_submit(
				&moonbeam_service,
				&mut submit_receiver,
				monitor_sender4.clone(),
			)
			.await.map_err(|e| (e.0, e.1.into()));
			if let Err(e) = res {
				if cfg!(feature = "monitor") {
					// let monitor_metrics = NotifyingMessage::new(
					// 	MOONBEAM_SUBMIT_LOG_TARGET.to_string(),
					// 	e.0,
					// 	&e.1,
					// 	config.keeper_address,
					// 	&config.moonbeam_client.ip_address,
					// );
					// monitor_sender4.send(monitor_metrics).await;
				}

				match e.1 {
					Error::MoonbeamError(MoonbeamError::Web3Error(_)) |
					Error::MoonbeamError(MoonbeamError::Web3ContractError(_)) |
					Error::TimeOutError(_) => {
						// todo need retry
						sleep().await;
						continue
					},
					_ => return e,
				};
			}
		}
	});

	// monitor
	let task_monitor_handle = tokio::spawn(async move {
		while let Some(msg) = monitor_receiver.recv().await {
			#[cfg(feature = "monitor")]
			{
				// let bot_url = &config.bot_url;
				// monitor::alert(&bot_url, msg.message().expect("monitor message parse wrong")).await;
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
