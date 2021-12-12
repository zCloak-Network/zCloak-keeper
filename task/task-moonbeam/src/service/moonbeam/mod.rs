use crate::{
	bus::MoonbeamTaskBus,
	config::{KiltConfig, MoonbeamConfig},
	message::MoonbeamTaskMessage,
	task::MoonbeamTask,
};
use component_ipfs::config::IpfsConfig;

use lifeline::{Bus, Lifeline, Receiver, Sender, Service, Task};
use server_traits::server::{config::Config, service::ServerService, task::ServerSand};

// use eth_keystore::decrypt_key;
// use secp256k1::SecretKey;

mod run;

#[derive(Debug)]
pub struct MoonBeamService {
	_greet: Lifeline,
}

impl ServerService for MoonBeamService {}

impl Service for MoonBeamService {
	type Bus = MoonbeamTaskBus;
	type Lifeline = anyhow::Result<Self>;

	fn spawn(bus: &Self::Bus) -> Self::Lifeline {
		let mut rx = bus.rx::<MoonbeamTaskMessage>()?;
		let tx = bus.tx::<MoonbeamTaskMessage>()?;
		let moonbean_config: MoonbeamConfig =
			Config::restore_with_namespace(MoonbeamTask::NAME, "moonbeam")?;
		let ipfs_config: IpfsConfig = Config::restore_with_namespace(MoonbeamTask::NAME, "ipfs")?;
		let kilt_config: KiltConfig = Config::restore_with_namespace(MoonbeamTask::NAME, "kilt")?;

		let _greet =
			Self::try_task(&format!("{}-service-task", MoonbeamTask::NAME), async move {
				while let Some(message) = rx.recv().await {
					let moonbean_config = moonbean_config.clone();
					let ipfs_config = ipfs_config.clone();
					let kilt_config = kilt_config.clone();
					let start = message.start_block.map(Into::into);
					let mut tx = tx.clone();
					tokio::spawn(async move {
						let r =
							run::run_worker(start, moonbean_config, ipfs_config, kilt_config).await;
						match r {
							Ok(()) => (),
							Err((restart, e)) => {
								use tokio::time::{sleep, Duration};

								let new_start = if restart == web3::types::U64::zero() {
									start
								} else {
									Some(restart)
								};

								log::warn!("[worker]meet an error: {:?}, restart after 1 sec from block:[{:}]", e, restart);
								sleep(Duration::from_secs(1)).await;
								// send new trigger msg
								tx.send(MoonbeamTaskMessage {
									start_block: new_start.map(|u| u.as_u64()),
								})
								.await.expect("send new message must success, or we panic to stop the process");
							},
						}
					});
				}
				Ok(())
			});
		Ok(Self { _greet })
	}
}
