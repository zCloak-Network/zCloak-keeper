use crate::{
	bus::MoonbeamTaskBus,
	config::{ContractConfig, KiltConfig, MoonbeamConfig},
	message::MoonbeamTaskMessage,
	task::MoonbeamTask,
};
use array_bytes::hex2bytes_unchecked as mybytes;
use component_ipfs::{client::IpfsClient, config::IpfsConfig};
use lifeline::{Bus, Lifeline, Receiver, Service, Task};
use primitives::utils::utils;
use server_traits::server::{config::Config, service::ServerService, task::ServerSand};
use std::path::{Path, PathBuf};
use web3::{
	contract::{Contract, Options},
	ethabi::{ethereum_types::U256, RawLog},
	futures::{future, StreamExt},
	signing::SecretKeyRef,
	transports::WebSocket,
	types::{
		Address, BlockId, BlockNumber, Bytes, FilterBuilder, TransactionParameters, H160, H256,
		U128,
	},
	Web3,
};

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
								user,
								ipfs_url,
								c_type,
								&program_hash,
								&proof_cid,
								&public_input,
								&public_output,
								expected_result,
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
	user: Address,
	ipfs_url: String,
	c_type: H256,
	program_hash: &[u8; 32],
	proof_cid: &[u8],
	public_input: &[u128],
	public_output: &[u128],
	expect_result: bool,
) -> anyhow::Result<()> {
	let ipfs_client = IpfsClient::new(ipfs_url);

	loop {
		//distaff verifier
		let res = utils::verifier_proof(
			String::from("moonbeam"),
			&ipfs_client,
			proof_cid,
			program_hash,
			public_input,
			public_output,
		)
		.await?;
	}

	Ok(())
}
