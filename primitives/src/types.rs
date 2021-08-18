use codec::{Decode, Encode};
use substrate_subxt::{
    sp_runtime::{MultiAddress,AccountId32},
    Runtime,EventTypeRegistry,
};

pub type BlockNumber = u32;
pub type Class = Vec<u8>;
pub type AssetId = u32;
pub type LookupSource = MultiAddress<AccountId32,()>;
pub type TAssetBalance = u128;

#[derive(Clone, Copy, Encode, Decode, PartialEq, Eq)]
enum TaskStatus {
    JustCreated,
    Verifying,
    VerifiedTrue,
    VerifiedFalse,
}

#[derive(Encode, Decode)]
enum UserTaskStatus {
    JustCreated,
    VerifiedTrue,
    VerifiedFalse,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub struct Status {
    verifiers: Vec<u32>,
    ayes: u32,
    nays: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub struct VerificationReceipt {
    program_hash: [u8; 32],
    passed: bool,
    submit_at: BlockNumber,
    auth_index: u32,
    validator_len: u32,
}

#[derive(PartialEq, Eq, Encode, Decode)]
pub struct TaskInfo {
    proof_id: Vec<u8>,
    inputs: Vec<u128>,
    outputs: Vec<u128>,
    program_hash: [u8; 32],
    is_task_finish: Option<TaskStatus>,
    expiration: Option<BlockNumber>,
}

