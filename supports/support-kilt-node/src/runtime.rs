// use codec::{Decode, Encode};
// use substrate_subxt::{
// 	balances::{AccountData, Balances},
// 	extrinsic::DefaultExtra,
// 	register_default_type_sizes, sp_core,
// 	sp_runtime::{
// 		generic::Header,
// 		traits::{BlakeTwo256, IdentifyAccount, Verify},
// 		AccountId32, MultiAddress, MultiSignature, OpaqueExtrinsic,
// 	},
// 	system::System,
// 	EventTypeRegistry, Runtime,
// };
// // use attestation::attestations::AttestationDetails;
// use crate::pallets::Attestation;

// pub type LookupSource = MultiAddress<AccountId32, ()>;
// pub type TAssetBalance = u128;
// type SessionIndex = u32;




// /// kilt Runtime
// #[derive(Debug, Clone, Eq, PartialEq)]
// pub struct KiltRuntime;
// impl Runtime for KiltRuntime {
// 	type Signature = MultiSignature;
// 	type Extra = DefaultExtra<Self>;

// 	fn register_type_sizes(registry: &mut EventTypeRegistry<Self>) {
// 		registry.register_type_size::<u128>("Balance");
// 		registry.register_type_size::<LookupSource>("LookupSource");
// 		registry.register_type_size::<TAssetBalance>("TAssetBalance");
// 		registry.register_type_size::<SessionIndex>("SessionIndex");

// 		registry.register_type_size::<Self::AccountId>("AccountId");
//         // registry.register_type_size::<AttestationDetails>("AttestationDetails");

// 		register_default_type_sizes(registry);
// 	}
// }

// impl Balances for KiltRuntime {
// 	type Balance = u128;
// }

// impl System for KiltRuntime {
// 	type Index = u32;
// 	type BlockNumber = u32;
// 	type Hash = sp_core::H256;
// 	type Hashing = BlakeTwo256;
// 	type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;
// 	type Address = MultiAddress<Self::AccountId, ()>;
// 	type Header = Header<Self::BlockNumber, BlakeTwo256>;
// 	type Extrinsic = OpaqueExtrinsic;
// 	type AccountData = AccountData<<Self as Balances>::Balance>;
// }

// impl Attestation for KiltRuntime{}
