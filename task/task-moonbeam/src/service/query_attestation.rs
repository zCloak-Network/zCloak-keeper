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
use component_ipfs::config::IpfsConfig;
use component_ipfs::client::IpfsClient;
use primitives::utils::utils;
use std::path::{Path, PathBuf};

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
        
		let _greet = Self::try_task(&format!("{}-service-task", MoonbeamTask::NAME), async move {
			while let Some(message) = rx.recv().await {
				match message {
					MoonbeamTaskMessage::IpfsProof(
                        AddProof {
                            user,
                            c_type,
                            program_hash,
                            public_input,
                            public_output,
                            proof_cid,
                            expected_result 
                        }) => {
                            let ipfs_url = ipfs_config.url_index.clone();
						    tokio::spawn(
                                async move { fetch_and_verify(user, ipfs_url, c_type, &program_hash, &proof_cid, &public_input, &public_output, expected_result).await });
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
    expect_result: bool
)->  anyhow::Result<()>  {    
    let ipfs_client = IpfsClient::new(ipfs_url);


    loop {
        //distaff verifier
        let res = utils::verifier_proof(
            String::from("moonbeam"),
            &ipfs_client,
            proof_cid,
            program_hash,
            public_input,
            public_output
        ).await?;
    }

    Ok(())
}

