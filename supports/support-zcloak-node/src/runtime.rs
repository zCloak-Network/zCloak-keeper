use codec::{Decode, Encode};
use substrate_subxt::{
    balances::{AccountData, Balances},
    extrinsic::DefaultExtra,
    sp_runtime::{
        MultiAddress,AccountId32, MultiSignature,
        traits::{BlakeTwo256, IdentifyAccount, Verify},
        OpaqueExtrinsic,        
        generic::Header,


    },
    register_default_type_sizes,sp_core,
    Runtime,EventTypeRegistry,
    system::System,
    BlockNumber,

};
use primitives::frame::verify::StarksVerifierSeperate;
use primitives::frame::sudo::Sudo;

pub type Class = Vec<u8>;
pub type AssetId = u32;
pub type LookupSource = MultiAddress<AccountId32, ()>;
pub type TAssetBalance = u128;
pub type ProgramHash = [u8; 32];
type SessionIndex = u32;

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

#[derive(Encode, Decode)]
pub struct Status {
    verifiers: Vec<u32>,
    ayes: u32,
    nays: u32,
}

#[derive(Encode, Decode)]
pub struct VerificationReceipt {
    program_hash: [u8; 32],
    passed: bool,
    submit_at: u32,
    auth_index: u32,
    validator_len: u32,
}

#[derive(Encode, Decode)]
pub struct TaskInfo {
    proof_id: Vec<u8>,
    inputs: Vec<u128>,
    outputs: Vec<u128>,
    program_hash: [u8; 32],
    is_task_finish: Option<TaskStatus>,
    expiration: Option<u32>,
}



/// zCloak Runtime
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ZcloakRuntime;
impl Runtime for ZcloakRuntime {

    type Signature = MultiSignature;
    type Extra = DefaultExtra<Self>;

    fn register_type_sizes(registry: &mut EventTypeRegistry<Self>) {
        registry.register_type_size::<Class>("Class");
        registry.register_type_size::<u128>("Balance");
        // registry.register_type_size::<AssetId>("AssetId");
        registry.register_type_size::<LookupSource>("LookupSource");
        registry.register_type_size::<TAssetBalance>("TAssetBalance");
        registry.register_type_size::<TaskStatus>("TaskStatus");
        registry.register_type_size::<UserTaskStatus>("UserTaskStatus");
        // registry.register_type_size::<Status>("Status");
        registry.register_type_size::<VerificationReceipt>("VerificationReceipt");
        registry.register_type_size::<TaskInfo>("TaskInfo");
        registry.register_type_size::<SessionIndex>("SessionIndex");
        registry.register_type_size::<Self::AccountId>("AccountId");
        registry.register_type_size::<[u8; 32]>("ProgramHash");
        registry.register_type_size::<[u8; 32]>("[u8; 32]");
        registry.register_type_size::<Self::BlockNumber>("BlockNumber");
        registry.register_type_size::<u32>("T::BlockNumber");

        register_default_type_sizes(registry);
    }

}


impl Balances for ZcloakRuntime {
    type Balance = u128;
}

impl System for ZcloakRuntime {
    type Index = u32;
    type BlockNumber = u32;
    type Hash = sp_core::H256;
    type Hashing = BlakeTwo256;
    type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;
    type Address = MultiAddress<Self::AccountId, ()>;
    type Header = Header<Self::BlockNumber, BlakeTwo256>;
    type Extrinsic = OpaqueExtrinsic;
    type AccountData = AccountData<<Self as Balances>::Balance>;
}

impl StarksVerifierSeperate for ZcloakRuntime {
    type ProgramHash = ProgramHash;
}
