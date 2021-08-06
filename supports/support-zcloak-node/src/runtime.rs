use codec::{Decode, Encode};
use substrate_subxt::{
    sp_runtime::MultiAddress,
    Runtime,EventTypeRegistry,BlockNumber,
};

pub type Class = Vec<u8>;
pub type AssetId = u32;
pub type LookupSource = MultiAddress;
pub type TAssetBalance = u128;

#[derive(Encode, Decode)]
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

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub struct TaskInfo {
    proof_id: Vec<u8>,
    inputs: Vec<u128>,
    outputs: Vec<u128>,
    program_hash: [u8; 32],
    is_task_finish: Option<TaskStatus>,
    expiration: Option<BlockNumber>,
}



/// zCloak Runtime
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ZcloakRuntime;
impl Runtime for ZcloakRuntime {

    fn register_type_sizes(registry: &mut EventTypeRegistry<Self>) {
        registry.register_type_size::<Class>("Class");
        registry.register_type_size::<AssetId>("AssetId");
        registry.register_type_size::<LookupSource>("LookupSource");
        registry.register_type_size::<TAssetBalance>("TAssetBalance");
        registry.register_type_size::<TaskStatus>("TaskStatus");
        registry.register_type_size::<UserTaskStatus>("UserTaskStatus");
        registry.register_type_size::<Status>("Status");
        registry.register_type_size::<VerificationReceipt>("VerificationReceipt");
        registry.register_type_size::<TaskInfo>("TaskInfo");
        register_default_type_sizes(registry);
    }

}


