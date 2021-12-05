use subxt::{
	sp_core::{sr25519::Pair, Pair as PairTrait},
	PairSigner,Config,
	sp_runtime::AccountId32,
};

pub struct KiltAccount {
	pub account_id: AccountId32,
	pub signer: PairSigner<Config, Pair>,
}

impl Clone for KiltAccount {
	fn clone(&self) -> Self {
		Self { account_id: self.account_id.clone(), signer: self.signer.clone() }
	}
}

impl KiltAccount {
	pub fn new(seed: String) -> KiltAccount {
		let pair = Pair::from_string(&seed, None).unwrap();
		let signer = PairSigner::new(pair);
		let public = signer.signer().public().0;
		let account_id = AccountId32::from(public);

		KiltAccount { account_id, signer }
	}
}
