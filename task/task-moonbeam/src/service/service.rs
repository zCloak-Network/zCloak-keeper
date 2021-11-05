use array_bytes::hex2bytes_unchecked as mybytes;
use lifeline::{Bus, Lifeline, Receiver, Service, Task};
use secp256k1::SecretKey;
use std::str::FromStr;
use anyhow::anyhow;
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
	bus::MoonbeamTaskBus, config::{MoonbeamConfig,ContractConfig}, message::MoonbeamTaskMessage, task::MoonbeamTask,
    event::CreateTaskEvent
};
use primitives::utils::ipfs::config::IpfsConfig;
use primitives::utils::ipfs::client::IpfsClient;
use primitives::utils::utils;
use server_traits::error::StandardError;
use starksVM as stark;
use codec::Decode;
use std::str;
use std::convert::TryInto;
use std::convert::TryFrom;








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
        let moonbean_config: MoonbeamConfig = Config::restore_with_namespace(MoonbeamTask::NAME, "moonbeam")?;
        let contract_config: ContractConfig = Config::restore_with_namespace(MoonbeamTask::NAME, "contract")?;
        let ipfs_config: IpfsConfig = Config::restore_with_namespace(MoonbeamTask::NAME, "ipfs")?;
        
        
		let _greet = Self::try_task(&format!("{}-service-task", MoonbeamTask::NAME), async move {
			while let Some(message) = rx.recv().await {
				match message {
					MoonbeamTaskMessage::TaskEvent => {

                        let url = moonbean_config.url.clone();
                        let address = contract_config.address.clone();
                        let topics = contract_config.topics.clone();
                        let ipfs_url = ipfs_config.url_index.clone();

						// zcloak_client.subscribe_events(ipfs_config.clone()).await?;
						tokio::spawn(async move { run_subscribe(url, address, topics, ipfs_url).await });
						log::info!("moonbeam server is running")
					},
				}
				log::debug!(
					target: MoonbeamTask::NAME,
					"[{}] recv a new task message: {:?}",
					MoonbeamTask::NAME,
					message
				);
			}
			Ok(())
		});
		Ok(Self { _greet })
	}
}

async fn run_subscribe(url: String, address: String, topics: Vec<String>, ipfs_url:String)->  anyhow::Result<()>  {
    // let url = moonbean_config.url.clone();
    // let address = contract_onfig.address.clone();
    // let topics = contract_onfig.topics.clone();
    log::info!("Moonbeam url is {:?}",&url);
    log::info!("The Contract deployed on Moonbeam , Address is {:?}", &address);

    let address = H160::from_slice(&mybytes(address));
    let topics = topics.iter().map(|t| H256::from_slice(&mybytes(t))).collect();
    
    let ipfs_client = IpfsClient::new(ipfs_url);

    let web3 = Web3::new(WebSocket::new(&url).await?);

    // if users don't known the topics ,we also can compute the call function to get the hex value;
    // let hash = web3::signing::keccak256("Transfer(address,address,uint256)".as_bytes());
    let hash = web3::signing::keccak256("CreateTaskEvent(address,bytes,uint128[],uint128[],string)".as_bytes());
    let hash = array_bytes::bytes2hex("", hash);
    log::info!("hash is {:?}", hash);

    let prvk = SecretKey::from_slice(&mybytes("0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133"))?;

    //get contract instance from json file
    let contract = Contract::from_json(
        web3.eth(),
        address.clone(),
        include_bytes!("./ZeroKnowlegeProof.json"),
    )?;

    let create_task_event = contract.abi().event("CreateTaskEvent").unwrap();
    let task_hash = create_task_event.signature();

    log::info!("create contract instance !");
    // get subscribtion
    let filter = FilterBuilder::default()
            .address(vec![address])
            .topics(
                Some(topics),
            None,
            None,
            None,
            )
            .build();
    log::info!("moonbeam service start to subscribe evm event!");
    let mut sub = web3.eth_subscribe().subscribe_logs(filter).await?;

    loop {
        let raw = sub.next().await;
        match raw {
            Some(event) =>{
                let event = event.unwrap();
                if event.topics[0] == task_hash {
                    let log = create_task_event.parse_log(RawLog {
                        topics: event.topics,
                        data: event.data.0,
                    });
                    match log {
                        Ok(log) => {
                            log::info!{"log is {:?}", &log};
                            let params = log.params;
                            let create_param = CreateTaskEvent::parse_log(params);

                            match create_param {
                                Ok(create_param) => {
                                    let res = utils::verifier_proof(
                                        String::from("moonbeam"),
                                        &ipfs_client,
                                        create_param.proof_id,
                                        create_param.program_hash,
                                        create_param.public_inputs,
                                        create_param.outputs
                                    ).await?;

                                    //call saveProof function transaction through contract instance
                                    let inputs = (create_param.sender,create_param.sender,create_param.program,res);

                                    let key_ref = SecretKeyRef::new(&prvk);

                                    let tx = contract.signed_call_with_confirmations(
                                        "saveProof",
                                        inputs, 
                                        Options::default(),
                                        1,
                                        key_ref,
                                    ).await?;

                                    log::info!("Summit proof resutl to ethereum with tx: {:?}",tx);

                                },
                                Err(e) => {
                                    log::error!("Parse params failed ! , exception stack is:{:?}", e);
                                }
                            }
                        }
                        _ => {
                            log::debug!{"moonbeam service : parse_log raw log failed !"}
                        }
                    }
                }
            },
            None => break,
        }
    }
    Ok(())
}

