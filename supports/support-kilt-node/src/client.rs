// use crate::{account::KiltAccount, runtime::KiltRuntime};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use server_traits::error::StandardError;
use subxt::{
	sp_core::{sr25519::Pair, Pair as PairTrait},
	sp_runtime::AccountId32,
	Client, ClientBuilder, Config, EventSubscription, PairSigner,
};

#[derive(Clone, Copy, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd, TypeInfo)]
pub enum DidEncryptionKey {
	/// An X25519 public key.
	X25519([u8; 32]),
}

#[subxt::subxt(runtime_metadata_path = "kilt_metadata.scale")]
pub mod kilt {
	#[subxt(substitute_type = "did::did_details::DidEncryptionKey")]
	use crate::kilt::DidEncryptionKey;
}

const _: () = {
	use kilt::runtime_types::polkadot_parachain::primitives::Id;

	impl PartialEq for Id {
		fn eq(&self, other: &Self) -> bool {
			self.0 == other.0
		}
	}

	impl Eq for Id {}

	impl PartialOrd for Id {
		fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
			self.0.partial_cmp(&other.0)
		}
	}

	impl Ord for Id {
		fn cmp(&self, other: &Self) -> std::cmp::Ordering {
			self.0.cmp(&other.0)
		}
	}
};

use kilt::runtime_types::did::did_details::DidDetails;

#[derive(Clone)]
pub struct Kilt {}

impl Kilt {
	pub async fn query_attestation(
		url: String,
		seed: String,
		root_hash: String,
	) -> anyhow::Result<()> {
		let pair = Pair::from_string(&seed, None).unwrap();
		let signer = PairSigner::<kilt::DefaultConfig, Pair>::new(pair);
		let public = signer.signer().public().0;
		let account_id = AccountId32::from(public);

		let api = ClientBuilder::new()
			.set_url(url)
			.build()
			.await?
			.to_runtime_api::<kilt::RuntimeApi<kilt::DefaultConfig>>();

		log::info!("------- query attestation ");
		let mut iter = api.storage().did().did_iter(None).await?;

		while let Some((key, DidDetails)) = iter.next().await? {
			log::info!("result is {:?}", DidDetails.last_tx_counter);
		}

		Ok(())
	}
}
