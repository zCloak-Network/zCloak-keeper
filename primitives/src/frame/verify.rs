use codec::{Decode, Encode};
use substrate_subxt::{
    system::System,
    Event, Encoded,
};
use substrate_subxt_proc_macro::{
    module, Call};

use std::fmt::Debug;
use crate::types::Class;
use crate::types::BlockNumber;
use core::marker::PhantomData;

#[module]
pub trait StarksVerifierSeperate: System {
    type ProgramHash: 'static + Encode + Decode + Sync + Send +Default;
}

#[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
pub struct UserTaskCreatedEvent<T: StarksVerifierSeperate> {
    pub who: <T as System>::AccountId,
    pub class: Class,
    pub programhash: T::ProgramHash,
    pub proofid: Vec<u8>,
    pub inputs: Vec<u128>,
    pub outputs: Vec<u128>,
}


#[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
pub struct WhiteListAddedEvent<T: StarksVerifierSeperate> {
    pub who: <T as System>::AccountId,
    pub block_number: BlockNumber,
}


#[derive(Clone, Debug, Eq, PartialEq, Call, Encode)]
pub struct ClientSingleReponseCall<T: StarksVerifierSeperate> {
    pub _runtime: PhantomData<T>,
    pub who: <T as System>::AccountId,
    pub class: Class,
    pub result: bool,
}

// #[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
// pub struct ClientSingleReponseEvent<T: StarksVerifierSeperate> {
//     pub who: <T as System>::AccountId,
//     pub class: Class,
//     pub result: bool,
// }




