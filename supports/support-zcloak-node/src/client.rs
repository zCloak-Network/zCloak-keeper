use crate::{account::ZcloakAccount, runtime::ZcloakRuntime};
use codec::Decode;
use primitives::{
	frame::verify::{ClientSingleReponseCallExt, UserTaskCreatedEvent},
	utils::ipfs::client::IpfsClient,
	utils::utils,
};
use server_traits::error::StandardError;
use sp_keyring::AccountKeyring;
use starksVM as stark;
use std::str;
use substrate_subxt::{
	balances::{TransferCallExt, TransferEvent},
	EventSubscription, PairSigner,
};
use subxt_client::SubstrateClient;

#[derive(Clone)]
pub struct Zcloak {
	pub client: SubstrateClient<ZcloakRuntime>,
	pub zcloak_account: ZcloakAccount,
}

impl Zcloak {
	pub fn new(client: SubstrateClient<ZcloakRuntime>, zcloak_account: ZcloakAccount) -> Self {
		Self { client, zcloak_account }
	}

	pub async fn subscribe_events(&self, ipfs_config: String) -> anyhow::Result<()> {
		log::info!("start to subscrible UserTaskCreatedEvent ---");
		let sub = self.client.subxt.subscribe_events().await?;
		// let decoder = self.client.subxt.events_decoder();
		let decoder = &self.client.event.decoder;
		let mut sub = EventSubscription::<ZcloakRuntime>::new(sub, &decoder);
		sub.filter_event::<UserTaskCreatedEvent<_>>();
		let ipfs_client = IpfsClient::new(ipfs_config);

		loop {
			let raw_event = sub.next().await.unwrap();

			match raw_event {
				Ok(r) => {
					log::info!("get raw event success --");

					if let Ok(e) = UserTaskCreatedEvent::<ZcloakRuntime>::decode(&mut &r.data[..]) {
						log::info!("start to para event data --");
						let who = e.who;
						let class = e.class;
						let programhash = e.programhash;
						let proofid = e.proofid;
						let inputs = e.inputs;
						let outputs = e.outputs;

						let res = utils::verifier_proof(
							String::from("zcloak node"),
							&ipfs_client,
							proofid,
							programhash,
							inputs,
							outputs
						).await?;

						log::debug! {"{:#?} commit a client single respnse call ---", &self.zcloak_account.account_id };
						
						let _ = &self
							.client
							.subxt
							.client_single_reponse(&self.zcloak_account.signer, who, class, res)
							.await?;

					} else {
						log::error!("decode row data error : {:?}", r);
					}
				},
				Err(e) => {
					log::error!("raw event get error : {:?}", e)
				},
			}
		}
		// Ok(());
	}

	pub async fn subscribe_transfer_events(&self) -> anyhow::Result<()> {
		log::info!("start to subscrible transfer event ---");
		let signer = PairSigner::new(AccountKeyring::Alice.pair());
		let dest = AccountKeyring::Bob.to_account_id().into();

		let sub = self.client.subxt.subscribe_events().await?;
		let decoder = self.client.subxt.events_decoder();
		let mut sub = EventSubscription::<ZcloakRuntime>::new(sub, decoder);
		sub.filter_event::<TransferEvent<_>>();
		self.client.subxt.transfer(&signer, &dest, 10_000).await?;

		loop {
			let raw_event = sub.next().await.unwrap();

			match raw_event {
				Ok(r) => {
					log::debug!("get raw event success ");

					let event = TransferEvent::<ZcloakRuntime>::decode(&mut &r.data[..]);
					if let Ok(e) = event {
						println!("Balance transfer success: value: {:?}", e.amount);
					} else {
						println!("Failed to subscribe to Balances::Transfer Event");
					}
				},
				Err(e) => {
					log::error!("raw event get error : {}", e)
				},
			}
		}
	}
}
