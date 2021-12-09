use crate::{
	bus::MoonbeamTaskBus,
	message::MoonbeamTaskMessage,
	task::MoonbeamTask,
};
use component_ipfs::{client::IpfsClient, config::IpfsConfig};
use lifeline::{Bus, Lifeline, Receiver, Service, Task};
use server_traits::server::{config::Config, service::ServerService, task::ServerSand};
use primitives::utils::utils::verifier_proof;
use crate::message::AddProof;

#[derive(Debug)]
pub struct IpfsService {
	_greet: Lifeline,
}

impl ServerService for IpfsService {}

impl Service for IpfsService {
	type Bus = MoonbeamTaskBus;
	type Lifeline = anyhow::Result<Self>;

	fn spawn(bus: &Self::Bus) -> Self::Lifeline {
		let mut rx = bus.rx::<MoonbeamTaskMessage>()?;
		let mut tx = bus.tx::<MoonbeamTaskMessage>()?;
		let ipfs_config: IpfsConfig = Config::restore_with_namespace(MoonbeamTask::NAME, "ipfs")?;

		let _greet = Self::try_task(&format!("{}-query-proof", MoonbeamTask::NAME), async move {
			while let Some(message) = rx.recv().await {
				match message {
					MoonbeamTaskMessage::IpfsProof(AddProof {
						user,
						c_type,
						program_hash,
						public_input,
						public_output,
						proof_cid,
						expected_result,
					}) => {
						let ipfs_url = ipfs_config.url_index.clone();
						tokio::spawn(async move {
							fetch_and_verify(
								ipfs_url,
								&program_hash,
								&proof_cid,
								&public_input,
								&public_output
							)
							.await
						});
						log::info!("moonbeam server is running")
					},

					// TODO: handle this
					_ => {},
				}
			}
			Ok(())
		});
		Ok(Self { _greet })
	}
}

// TODO: move ipfs connection out of verify_proof
async fn fetch_and_verify(
	ipfs_url: String,
	program_hash: &[u8; 32],
	proof_cid: &[u8],
	public_input: &[u128],
	public_output: &[u128],
) -> anyhow::Result<bool> {
	let ipfs_client = IpfsClient::new(ipfs_url);
	let mut res = false;
	while let Ok(body) = ipfs_client.keep_fetch_proof(proof_cid).await {
		//distaff verifier
		res = verifier_proof(
			program_hash,
			body,
			public_input,
			public_output
		)?;
	}

	Ok(res)
}
