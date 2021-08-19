use codec::Encode;
use core::marker::PhantomData;
use substrate_subxt::{system::System, Encoded};

use frame_support::weights::Weight;
use substrate_subxt_proc_macro::{module, Call};

#[module]
pub trait Sudo: System {}

#[derive(Clone, Debug, Eq, PartialEq, Call, Encode)]
pub struct SudoCall<'a, T: Sudo> {
	pub _runtime: PhantomData<T>,
	pub call: &'a Encoded,
}

#[derive(Clone, Debug, Eq, PartialEq, Call, Encode)]
pub struct SudoUncheckedWeightCall<'a, T: Sudo> {
	pub _runtime: PhantomData<T>,
	pub call: &'a Encoded,
	pub weight: Weight,
}
