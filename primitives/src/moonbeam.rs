use std::os::unix::prelude::OsStringExt;
use web3::contract::{Error as Web3ContractErr, Contract, tokens::{Tokenize, Detokenize} };
pub use super::*;
use web3::{self as web3, api::Eth, transports::Http, ethabi, Transport };
use crate::error::Error;

pub const MOONBEAM_SCAN_SPAN: usize = 10;
// TODO: move it to config file
pub const MOONBEAM_LISTENED_EVENT: &'static str = "AddProof";
pub const MOONBEAM_BLOCK_DURATION: u64 = 12;
pub const MOONBEAM_TRANSACTION_CONFIRMATIONS: usize = 2;

pub type Result<T> = std::result::Result<T, Error>;
pub type ProofEventType = (Address, Bytes32, Bytes32, Bytes32, String, String, Bytes32, bool);


#[derive(Debug, Default)]
pub struct ProofEvent {
    pub(crate) data_owner: Address,
    pub(crate) kilt_address: Bytes32,
    pub(crate) c_type: Bytes32,
    pub(crate) program_hash: Bytes32,
    pub(crate) field_name: String,
    pub(crate) proof_cid: String,
    pub(crate) root_hash: Bytes32,
    pub(crate) expect_result: bool,
}

impl ProofEvent {
    pub fn proof_cid(&self) -> &str {
        self.proof_cid.as_str()
    }
    pub fn public_inputs(&self) -> Vec<u128> {
        let hex_str = hex::encode(&self.field_name);
        let r = u128::from_str_radix(&hex_str, 16)
            .expect("filed_name from event must be fit into u128 range");
        // TODO in future, other params can be part of the inputs
        vec![r]
    }

    // calc the output from `ProofEvent`
    pub fn outputs(&self) -> Vec<u128> {
        let mut outputs = vec![];
        let mut mid: [u8; 16] = Default::default();
        mid.copy_from_slice(&self.root_hash[0..16]);
        outputs.push(u128::from_be_bytes(mid));
        mid.copy_from_slice(&self.root_hash[16..]);
        outputs.push(u128::from_be_bytes(mid));
        if self.expect_result {
            outputs.push(1)
        } else {
            outputs.push(0)
        }

        outputs
    }
    pub fn program_hash(&self) -> Bytes32 {
        self.program_hash
    }
}


impl From<ProofEventType> for ProofEvent {
    fn from(tuple_type: ProofEventType) -> Self {
        Self {
            data_owner: tuple_type.0,
            kilt_address: tuple_type.1,
            c_type: tuple_type.2,
            program_hash: tuple_type.3,
            field_name: tuple_type.4,
            proof_cid: tuple_type.5,
            root_hash: tuple_type.6,
            expect_result: tuple_type.7,
        }
    }
}

// TODO: transform
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MoonbeamConfig {
    pub url: String,
    // where users add their proofs and emit `AddProof` event
    pub read_contract: String,
    // where keeper submit the verify result
    pub write_contract: String,
    pub private_key: String,
}

pub struct MoonbeamClient {
    connect: Web3<Http>
}


impl MoonbeamClient {

    fn connect(url: &str) -> Result<Self> {
        let http = web3::transports::Http::new(url)?;
        Ok(Self {connect: Web3::new(http) })
    }


    fn proof_contract(&self, proof_addr: &str) -> Result<Contract<Http>> {
        let addr = if proof_addr.starts_with("0x") {
            &proof_addr[2..]
        } else {
            proof_addr
        };

        let hex_res =
            hex::decode(addr).map_err(|e| Error::InvalidEthereumAddress(format!("{:}", e)))?;
        if hex_res.len() != 20 {
            return Err(Error::InvalidEthereumAddress(format!(
                "Address is not equal to 20 bytes: {:}",
                addr
            )))
        }
        let address = Address::from_slice(&hex_res);

        let kilt_proofs_v1 = Contract::from_json(
            self.connect.eth(),
            address,
            include_bytes!("../contracts/KiltProofs.json"),
        )?;
        Ok(kilt_proofs_v1)
    }
}

pub mod utils {
    use super::*;
    pub async fn events<T: Transport, R: Detokenize>(
        web3: Eth<T>,
        contract: &Contract<T>,
        event: &str,
        from: Option<U64>,
        to: Option<U64>,
    ) -> std::result::Result<Vec<(R, Log)>, Web3ContractErr> {
        fn to_topic<A: Tokenize>(x: A) -> ethabi::Topic<ethabi::Token> {
            let tokens = x.into_tokens();
            if tokens.is_empty() {
                ethabi::Topic::Any
            } else {
                tokens.into()
            }
        }

        let res = contract.abi().event(event).and_then(|ev| {
            let filter = ev.filter(ethabi::RawTopicFilter {
                topic0: to_topic(()),
                topic1: to_topic(()),
                topic2: to_topic(()),
            })?;
            Ok((ev.clone(), filter))
        });
        let (ev, filter) = match res {
            Ok(x) => x,
            Err(e) => return Err(e.into()),
        };

        let mut builder = FilterBuilder::default().topic_filter(filter);
        if let Some(f) = from {
            builder = builder.from_block(BlockNumber::Number(f));
        }
        if let Some(t) = to {
            builder = builder.to_block(BlockNumber::Number(t));
        }

        let filter = builder.build();

        let logs = web3.logs(filter).await?;
        logs.into_iter()
            .map(move |l| {
                let log = ev.parse_log(ethabi::RawLog {
                    topics: l.topics.clone(),
                    data: l.data.0.clone(),
                })?;

                Ok((
                    R::from_tokens(log.params.into_iter().map(|x| x.value).collect::<Vec<_>>())?,
                    l,
                ))
            })
            .collect::<_>()
    }
}
