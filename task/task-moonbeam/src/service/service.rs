use array_bytes::hex2bytes_unchecked as mybytes;
use lifeline::{Bus, Lifeline, Receiver, Service, Task};
use secp256k1::SecretKey;
use std::str::FromStr;

use web3::{
    Web3,
    ethabi::ethereum_types::U256,
    transports::WebSocket,
    futures::{future, StreamExt},
    types::{FilterBuilder, H160, H256, U128, TransactionParameters, Bytes},
    contract::{Contract, Options}
};
use server_traits::server::{config::Config, service::ServerService, task::ServerSand};
use crate::{
	bus::MoonbeamTaskBus, config::{MoonbeamConfig,ContractConfig}, message::MoonbeamTaskMessage, task::MoonbeamTask,
};



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
        
        
		let _greet = Self::try_task(&format!("{}-service-task", MoonbeamTask::NAME), async move {
			while let Some(message) = rx.recv().await {
				match message {
					MoonbeamTaskMessage::TaskEvent => {

                        let url = moonbean_config.url.clone();
                        let address = contract_config.address.clone();
                        let topics = contract_config.topics.clone();
						// zcloak_client.subscribe_events(ipfs_config.clone()).await?;
						tokio::spawn(async move { run_subscribe(url, address, topics).await });
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

async fn run_subscribe(url: String, address: String, topics: Vec<String>)->  anyhow::Result<()>  {
    // let url = moonbean_config.url.clone();
    // let address = contract_onfig.address.clone();
    // let topics = contract_onfig.topics.clone();
    log::info!("Moonbeam url is {:?}",&url);
    log::info!("The Contract deployed on Moonbeam , Address is {:?}", &address);

    let address = H160::from_slice(&mybytes(address));
    let topics = topics.iter().map(|t| H256::from_slice(&mybytes(t))).collect();
    

    let web3 = Web3::new(WebSocket::new(&url).await?);
    // if users don't known the topics ,we also can compute the call function to get the hex value;
    // let hash = web3::signing::keccak256("Transfer(address,address,uint256)".as_bytes());
    // let hash = array_bytes::bytes2hex("", hash);
    // log::info!("hash is {:?}", hash);

    //get contract instance from json file
    let contract = Contract::from_json(
        web3.eth(),
        address,
        include_bytes!("MyToken.json"),
    )?;

    let from = H160::from_slice(&mybytes("0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac"));

    // let program_hash = Bytes(vec!(1));
    // let public_input = U128::from(2);
    // let output = U128::from(3);
    // let proof_id = "0xdfsfd";

    // let tx = contract::<Bytes, U128[], U128[], U8>().call("CreateTask",(program_hash,public_input,output,proof_id,),from, Options::default()).await?;





    //do transaction 
    let prvk = SecretKey::from_slice(&mybytes("0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133"))?;
    let to = H160::from_slice(&mybytes("0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0"));
    let tx_object = TransactionParameters {
        to: Some(to),
        value: U256::exp10(18),
        ..Default::default()
    };

    let signed = web3.accounts().sign_transaction(tx_object, &prvk).await?;
    let result = web3.eth().send_raw_transaction(signed.raw_transaction).await?;

    log::info!("TX succeeded with hash: {}", result);
    
    

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

    let sub = web3.eth_subscribe().subscribe_logs(filter).await?;
    sub.for_each(|log| {
        log::info!("evm contract event log context: {:?}", log);
        future::ready(())
    }).await;

    Ok(())
}
