// use codec::{Decode, Encode};
// use substrate_subxt::{system::System, Event};
// use substrate_subxt::sp_core::H256;
// use substrate_subxt_proc_macro::{module, Store, Call};

// use core::marker::PhantomData;
// use std::fmt::Debug;
// // use attestation::attestations::AttestationDetails
// // use frame_support::Parameter;
// use scale_info::TypeInfo;


// #[module]
// pub trait Attestation: System {
//     // type DelegationNodeId: Parameter + Copy + AsRef<[u8]> + Eq + PartialEq + Ord + PartialOrd ;
//     // type DelegationEntityId: Parameter + TypeInfo;
// }

// #[derive(Clone, Debug, Eq, PartialEq, Store, Decode, Encode)]
// pub struct Attestations<T: Attestation> {
//     #[store(returns = String)]
//     /// Runtime marker.
//     pub _runtime: PhantomData<T>,
// }

// // pub type AttestationDetails<T> = (
// //     H256,
// //     <T as Attestation>::DelegationEntityId,
// //     Option<<T as Attestation>::DelegationNodeId>,
// //     bool,
// //     Deposit<<T as System>::AccountId, <T as Balances>::Balance>
// // );


// // #[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo)]
// // pub struct Deposit<Account, Balance> {
// // 	pub owner: Account,
// // 	pub amount: Balance,
// // }