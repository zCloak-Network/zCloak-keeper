// use codec::{Decode, Encode};
// use subxt::{system::System, Event};
// use subxt_macro::{module, Call};

// use crate::types::{BlockNumber, Class};
// use core::marker::PhantomData;
// use std::fmt::Debug;

// #[module]
// pub trait StarksVerifierSeperate: System {
// 	type ProgramHash: 'static + Encode + Decode + Sync + Send + Default;
// }

// #[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
// pub struct UserTaskCreatedEvent<T: StarksVerifierSeperate> {
// 	pub who: <T as System>::AccountId,
// 	pub class: Class,
// 	pub programhash: T::ProgramHash,
// 	pub proofid: Vec<u8>,
// 	pub inputs: Vec<u128>,
// 	pub outputs: Vec<u128>,
// }

// #[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
// pub struct WhiteListAddedEvent<T: StarksVerifierSeperate> {
// 	pub who: <T as System>::AccountId,
// 	pub block_number: BlockNumber,
// }

// #[derive(Clone, Debug, Eq, PartialEq, Call, Encode)]
// pub struct ClientSingleReponseCall<T: StarksVerifierSeperate> {
// 	pub _runtime: PhantomData<T>,
// 	pub who: <T as System>::AccountId,
// 	pub class: Class,
// 	pub result: bool,
// }

// // #[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
// // pub struct ClientSingleReponseEvent<T: StarksVerifierSeperate> {
// //     pub who: <T as System>::AccountId,
// //     pub class: Class,
// //     pub result: bool,
// // }
