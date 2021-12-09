use array_bytes::hex2bytes_unchecked as mybytes;
use lifeline::{Bus, Lifeline, Receiver, Service, Task};
use web3::{
    Web3,
    ethabi::ethereum_types::U256,
    ethabi::RawLog,
    transports::WebSocket,
    futures::{future, StreamExt},
    types::{FilterBuilder, H160, H256, U128, TransactionParameters, Bytes, BlockId, BlockNumber, Address},
    contract::{Contract, Options},
    signing::SecretKeyRef

};
use server_traits::server::{config::Config, service::ServerService, task::ServerSand};
use crate::{
	bus::MoonbeamTaskBus, config::{MoonbeamConfig,ContractConfig,KiltConfig}, 
    message::MoonbeamTaskMessage, task::MoonbeamTask
};
use primitives::utils::utils;
use std::path::{Path, PathBuf};
use crate::message::KiltAttestation;
use support_kilt_node::client::query_attestation;


#[derive(Debug)]
pub struct AttestationService {
	_greet: Lifeline,
}

impl ServerService for AttestationService {}

impl Service for IpfsService {
	type Bus = MoonbeamTaskBus;
	type Lifeline = anyhow::Result<Self>;

	fn spawn(bus: &Self::Bus) -> Self::Lifeline {
		let mut rx = bus.rx::<MoonbeamTaskMessage>()?;
        let mut tx = bus.tx::<MoonbeamTaskMessage>()?;
        let kilt_config: IpfsConfig = Config::restore_with_namespace(MoonbeamTask::NAME, "kilt")?;
        
		let _greet = Self::try_task(
            &format!("{}-query-attestation", MoonbeamTask::NAME), 
            async move {
			    while let Some(message) = rx.recv().await {
				    match message {
					    MoonbeamTaskMessage::KiltAttestation(attestation) => {
                                let kilt_url = kilt_config.url.clone();
                                let is_valid = query_attestation(
                                    ipfs_url, 
                                    attestation.root_hash
                                ).await?;

                                if is_valid {
                                    tx.send(MoonbeamTaskMessage::SubmitVerification(attestation)).await?;
                                }
					}, 
                    _ => continue,
                }
			}
			Ok(())
		});
		Ok(Self { _greet })
	}
}