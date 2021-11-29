// use crate::runtime::ZcloakRuntime;
// use substrate_subxt::{
// 	sp_core::{sr25519::Pair, Pair as PairTrait},
// 	system::System,
// 	PairSigner,
// };

// pub type AccountId = <ZcloakRuntime as System>::AccountId;

// pub struct ZcloakAccount {
// 	pub account_id: AccountId,
// 	pub signer: PairSigner<ZcloakRuntime, Pair>,
// }

// impl Clone for ZcloakAccount {
// 	fn clone(&self) -> Self {
// 		Self { account_id: self.account_id.clone(), signer: self.signer.clone() }
// 	}
// }

// impl ZcloakAccount {
// 	pub fn new(seed: String) -> ZcloakAccount {
// 		let pair = Pair::from_string(&seed, None).unwrap();
// 		let signer = PairSigner::<ZcloakRuntime, Pair>::new(pair);
// 		let public = signer.signer().public().0;
// 		let account_id = AccountId::from(public);

// 		ZcloakAccount { account_id, signer }
// 	}
// }
