use std::{str::FromStr, sync::Arc, time::Duration};

use futures_timer::Delay;
use log::log;
use secp256k1::SecretKey;
use tokio::{io, sync::RwLock};
use yaque::{channel, recovery};

use keeper_primitives::{
	config::Error as ConfigError, ipfs::IPFS_LOG_TARGET, kilt::KILT_LOG_TARGET,
	moonbeam::MOONBEAM_LOG_TARGET, verify::VERIFY_LOG_TARGET, Config, Contract, Error, EventResult,
	Http, IpfsClient, JsonParse, KiltClient, MoonbeamClient, Result, VerifyResult, U64,
};

use crate::command::StartOptions;

// TODO: move to config
const CHANNEL_LOG_TARGET: &str = "Channel";
const MESSAGE_PARSE_LOG_TARGET: &str = "Message Parse";

const EVENT_TO_IPFS_CHANNEL: &str = "./data/event2ipfs";
const VERIFY_TO_ATTEST_CHANNEL: &str = "./data/verify2attest";
const ATTEST_TO_SUBMIT_CHANNEL: &str = "./data/attest2submit";

#[derive(Clone, Debug)]
pub struct ConfigInstance {
	pub(crate) moonbeam_client: MoonbeamClient,
	pub(crate) ipfs_client: IpfsClient,
	pub(crate) kilt_client: KiltClient,
	pub(crate) proof_contract: Contract<Http>,
	pub(crate) aggregator_contract: Contract<Http>,
	pub(crate) private_key: SecretKey,
}

pub async fn start(start_options: StartOptions) -> std::result::Result<(), Error> {
	// load config
	let start: U64 = start_options.start_number.unwrap_or_default().into();
	dbg!(start);
	let config_path = start_options.config.ok_or::<Error>(
		ConfigError::OtherError("Config File need to be specific".to_owned()).into(),
	)?;
	let config = Config::load_from_json(&config_path)?;

	log::info!("[Config] load successfully!");
	// init config
	let moonbeam_client = MoonbeamClient::new(config.moonbeam.url)?;
	let ipfs_client = IpfsClient::new(&config.ipfs.base_url)?;
	let kilt_client = KiltClient::try_from_url(&config.kilt.url).await?;

	let proof_contract = moonbeam_client.proof_contract(&config.moonbeam.read_contract)?;
	let aggregator_contract =
		moonbeam_client.aggregator_contract(&config.moonbeam.write_contract)?;

	let moonbeam_worker_pri = secp256k1::SecretKey::from_str(&config.moonbeam.private_key)?;

	let config_instance = ConfigInstance {
		moonbeam_client,
		ipfs_client,
		kilt_client,
		proof_contract,
		aggregator_contract,
		private_key: moonbeam_worker_pri,
	};

	// run a keeper
	run(start, Arc::new(RwLock::new(config_instance))).await;

	Ok(())
}

// handle detailed process
pub async fn run(start: U64, configs: Arc<RwLock<ConfigInstance>>) -> std::result::Result<(), keeper_primitives::Error> {
	// it record the latest block that contains proofevents
	// used in ganache
	let mut start = start;
	// force recover all channels, which delete all '.lock' files
	recovery::unlock_queue(EVENT_TO_IPFS_CHANNEL);
	recovery::unlock_queue(VERIFY_TO_ATTEST_CHANNEL);
	recovery::unlock_queue(ATTEST_TO_SUBMIT_CHANNEL);

	let (mut event_sender, mut event_receiver) = channel(EVENT_TO_IPFS_CHANNEL).unwrap();
	let (mut attest_sender, mut attest_receiver) = channel(VERIFY_TO_ATTEST_CHANNEL).unwrap();
	let (mut submit_sender, mut submit_receiver) = channel(ATTEST_TO_SUBMIT_CHANNEL).unwrap();

	// spead configs
	let config1 = configs.clone();
	let config2 = configs.clone();
	let config3 = configs.clone();
	let config4 = configs.clone();

	// 1. scan moonbeam proof event, and push them to event channel
	let task_scan = tokio::spawn(async move {
		let mut tmp_start_cache = 0.into();
		let config = config1.read().await;
		loop {

			let maybe_best = config.moonbeam_client.best_number().await;
			let best = match maybe_best {
				Ok(b) => b,
				Err(e) => {
					log::error!(
						target: MOONBEAM_LOG_TARGET,
						"Fail to get latest block number in task moonbeam scan, after #{:?} scanned",
						start
					);
					continue
				},
			};

			// local network check
			// only work if the chain is frozen
			if (start == tmp_start_cache) && (start == best) {
				// do nothing here
				continue
			}

			let res;
			let end;
			match moonbeam::scan_events(
				start,
				best,
				&config.moonbeam_client,
				&config.proof_contract,
			)
			.await
			{
				Ok(r) => {
					res = r.0;
					end = r.1
				},
				Err(e) => {
					// repeat scanning from the start again
					start = e.0;
					continue
				},
			}

			if !res.is_empty() {
				// send result to channel
				let output = res.into_bytes().expect("proofs encode into bytes error in task moonbeam scan");

				let status = event_sender.send(output).await;
				if let Err(_) = status {
					log::error!(
						target: CHANNEL_LOG_TARGET,
						"Fail to write data in block from: #{:?} into event channel file",
						start,
					);
					// repeat scanning from the start again
					continue
				}
				// After the proofevent list successfully sent to task2
				// reset the tmp_start_cache
				tmp_start_cache = end;
			} else {
				let latest = &config.moonbeam_client.best_number().await.unwrap_or_default();
				if start == *latest {
					// if current start is the best number, then sleep the block duration.
					use tokio::time::{sleep, Duration};
					log::info!("sleep for scan block... current:{:}|best:{:}", start, latest);
					sleep(Duration::from_secs(
						keeper_primitives::moonbeam::MOONBEAM_BLOCK_DURATION,
					))
					.await;
				}
				// continue;
			}

			// reset scan start point
			start = end;
		}
	});

	// 2. query ipfs and verify cid proof
	// TODO: separate ipfs query end starksvm verify
	let task_ipfs_verify = tokio::spawn(async move {
		let config = config2.read().await;

		while let Ok(events) = event_receiver.recv_timeout(Delay::new(Duration::from_secs(1))).await
		{
			let events = match events {
				Some(a) => a,
				None => continue,
			};

			// parse event from str to ProofEvent
			let inputs = EventResult::try_from_bytes(&*events);
			let inputs = match inputs {
				Ok(r) => r,
				Err(e) => {
					// log error
					log::error!(
						target: MESSAGE_PARSE_LOG_TARGET,
						"event messages in ipfs component wrongly parsed, {:?}",
						e
					);
					continue
				},
			};

			let r = ipfs::query_and_verify(&config.ipfs_client, inputs).await;
			let res = match r {
				Ok(v) => v,
				Err(e) => continue,
			};
			let status = attest_sender.send(serde_json::to_vec(&res).unwrap()).await;

			match status {
				Ok(_) => {
					// delete events in channel after the events are successfully
					// transformed and pushed into
					events.commit().expect("not commit in task ipfs_and_verify");
				},
				Err(e) => {
					log::error!("in task2 send to queue error:{:?}", e);
					continue
				},
			}
		}
	});

	//
	// 3. query kilt
	let task_kilt_attest = tokio::spawn(async move {
		let config = config3.read().await;

		while let Ok(r) = attest_receiver.recv_timeout(Delay::new(Duration::from_secs(1))).await {
			// while let Ok(events) = event_receiver.recv().await {
			let r = match r {
				Some(a) => a,
				None => continue,
			};
			log::info!("recv msg in task3");
			// parse verify result from str to VerifyResult
			let inputs = serde_json::from_slice(&*r).expect("serde json error in task attestation");

			let res = kilt::filter(&config.kilt_client, inputs).await;
			let verify_res = match res {
				Ok(r) => r,
				Err(_) => continue,
			};

			if !verify_res.is_empty() {
				let message_to_send = serde_json::to_vec(&verify_res);
				let status = submit_sender.send(message_to_send.unwrap()).await;

				match status {
					Ok(_) => {
						r.commit().expect("msg not commit in task attestation");
					},
					Err(e) => continue,
				}
			}
		}
	});

	// 4. submit tx
	let task_submit_tx = tokio::spawn(async move {
		let config = config4.read().await;

		while let Ok(r) = submit_receiver.recv_timeout(Delay::new(Duration::from_secs(1))).await {
			// while let Ok(events) = event_receiver.recv().await {
			let r = match r {
				Some(a) => a,
				None => continue,
			};
			log::info!("recv msg in task4");

			let inputs = serde_json::from_slice(&*r).expect("message decode error in task moonbeam submission");

			let res =
				moonbeam::submit_txs(&config.aggregator_contract, config.private_key, inputs).await;
			match res {
				Ok(_) => {
					r.commit().expect("fail to commit in task moonbeam submission");
				},
				Err(e) => continue,
			};
		}
	});

	tokio::try_join!(task_scan, task_ipfs_verify, task_kilt_attest, task_submit_tx)?;
	Ok(())
}
