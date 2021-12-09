use crate::{
	bus::MoonbeamTaskBus,
	config::{ContractConfig, KiltConfig, MoonbeamConfig},
	event::CreateTaskEvent,
	message::MoonbeamTaskMessage,
	task::MoonbeamTask,
};
use array_bytes::hex2bytes_unchecked as mybytes;
use component_ipfs::{config::IpfsConfig, IpfsClient};
use eth_keystore::decrypt_key;
use lifeline::{Bus, Lifeline, Receiver, Service, Task};
use primitives::utils::utils;
use secp256k1::SecretKey;
use server_traits::server::{config::Config, service::ServerService, task::ServerSand};
use std::path::Path;
use web3::{
	contract::{Contract, Options},
	ethabi::RawLog,
	futures::StreamExt,
	signing::SecretKeyRef,
	transports::WebSocket,
	types::{
		FilterBuilder, H160, H256,
	},
	Web3,
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
		let moonbean_config: MoonbeamConfig =
			Config::restore_with_namespace(MoonbeamTask::NAME, "moonbeam")?;
		let contract_config: ContractConfig =
			Config::restore_with_namespace(MoonbeamTask::NAME, "contract")?;
		let ipfs_config: IpfsConfig = Config::restore_with_namespace(MoonbeamTask::NAME, "ipfs")?;
		let kilt_config: KiltConfig = Config::restore_with_namespace(MoonbeamTask::NAME, "kilt")?;

		let _greet = Self::try_task(&format!("{}-service-task", MoonbeamTask::NAME), async move {
			while let Some(message) = rx.recv().await {
				match message {
					MoonbeamTaskMessage::ListenMoonbeam => {
						let url = moonbean_config.url.clone();
						let address = contract_config.address.clone();
						let topics = contract_config.topics.clone();
						let ipfs_url = ipfs_config.url_index.clone();
						let password = contract_config.password.clone();
						let uuid = contract_config.uuid.clone();
						let kilt_url = kilt_config.url.clone();
						let seed = kilt_config.private_key.clone();

						// zcloak_client.subscribe_events(ipfs_config.clone()).await?;
						tokio::spawn(async move {
							run_subscribe(
								url, address, topics, ipfs_url, password, uuid, kilt_url, seed,
							)
							.await
						});
						log::info!("moonbeam server is running")
					},
					//TODO: fill this later
					MoonbeamTaskMessage::SubmitVerification(attestation) => {},
					_ => continue,
				}
			}
			Ok(())
		});
		Ok(Self { _greet })
	}
}

async fn run_subscribe(
	url: String,
	address: String,
	topics: Vec<String>,
	ipfs_url: String,
	password: String,
	uuid: String,
	kilt_url: String,
	seed: String,
) -> anyhow::Result<()> {
	// let url = moonbean_config.url.clone();
	// let address = contract_onfig.address.clone();
	// let topics = contract_onfig.topics.clone();
	log::info!("Moonbeam url is {:?}", &url);
	log::info!("The Contract deployed on Moonbeam , Address is {:?}", &address);

	let address = H160::from_slice(&mybytes(address));
	let topics = topics.iter().map(|t| H256::from_slice(&mybytes(t))).collect();

	let ipfs_client = IpfsClient::new(ipfs_url);

	let web3 = Web3::new(WebSocket::new(&url).await?);

	//get contract instance from json file
	let contract =
		Contract::from_json(web3.eth(), address.clone(), include_bytes!("../../contracts/KiltProofs.json"))?;

	log::info!("create contract instance !");

	let create_task_event = contract.abi().event("AddProof").unwrap();
	let task_hash = create_task_event.signature();

	let mut file = String::from("/Users/jay/Project/zCloak-worker/keys/");
	file.push_str(uuid.as_str());

	let key_path = Path::new(&file);
	let entropy = decrypt_key(&key_path, password)?;
	let prk = hex::encode(entropy);
	let prvk = SecretKey::from_slice(&mybytes(prk))?;

	// get subscribtion
	let filter = FilterBuilder::default()
		.address(vec![address])
		.topics(Some(topics), None, None, None)
		.build();
	log::info!("moonbeam service start to subscribe evm event!");
	let mut sub = web3.eth_subscribe().subscribe_logs(filter).await?;

	loop {
		let raw = sub.next().await;
		match raw {
			Some(event) => {
				let event = event.unwrap();
				if event.topics[0] == task_hash {
					let log = create_task_event
						.parse_log(RawLog { topics: event.topics, data: event.data.0 });
					match log {
						Ok(log) => {
							log::info! {"log is {:?}", &log};
							let params = log.params;
							let create_param = CreateTaskEvent::parse_log(params);

							match create_param {
								Ok(create_param) => {
									//distaff verifier
									// let res = utils::verifier_proof(
									// 	&create_param.program_hash,
									// 	&create_param.public_inputs,
									// 	&create_param.outputs,
									// )
									// .await?;

									//kilt storage get
									log::info!("kilt 0-----{:?}", kilt_url.clone());
									let root_hash = "".to_string();

									// let attestations = Kilt::query_attestation(
									// 	kilt_url.clone(),
									// 	seed.clone(),
									// 	root_hash,
									// )
									// .await?;
									// log::info!("kilt 1-----");

									//call saveProof function transaction through contract instance
									let inputs = (
										create_param.sender,
										create_param.root_hash,
										create_param.c_type,
										create_param.program,
										true,
										true,
									);

									let key_ref = SecretKeyRef::new(&prvk);

									let tx = contract
										.signed_call_with_confirmations(
											"addVerification",
											inputs,
											Options::default(),
											1,
											key_ref,
										)
										.await?;

									log::info!("Summit proof resutl to ethereum with tx: {:?}", tx);
								},
								Err(e) => {
									log::error!("Parse params failed ! , {:?}", e);
									continue
								},
							}
						},
						_ => {
							log::debug! {"moonbeam service : parse_log raw log failed !"}
							continue
						},
					}
				}
			},
			None => break,
		}
	}
	Ok(())
}
