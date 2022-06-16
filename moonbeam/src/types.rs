use std::str::FromStr;
use secp256k1::SecretKey;
use web3::{
    self as web3,
    Web3,
    api::Eth,
    contract::{
        tokens::{Detokenize, Tokenize},
        Contract, Error as Web3ContractErr,
        Options as Web3Options,
    },
    ethabi,
    transports::Http,
    Transport,
    types::{Address, BlockNumber, FilterBuilder, Log, U64},
};
use super::error::Error;
use keeper_primitives::{Bytes32, keeper::KeeperSetting, traits::IpAddress};
use prometheus_endpoint::Registry;
use crate::metrics::MoonbeamMetrics;
use super::{Deserialize, Serialize};

pub const SUBMIT_TX_MAX_RETRY_TIMES: usize = 3;
pub const MOONBEAM_SCAN_SPAN: usize = 10;
// TODO: move it to config file
pub const MOONBEAM_LISTENED_EVENT: &'static str = "AddProof";
pub const MOONBEAM_BLOCK_DURATION: u64 = 12;
pub const MOONBEAM_TRANSACTION_CONFIRMATIONS: usize = 2;
pub const MOONBEAM_SCAN_LOG_TARGET: &str = "MoonbeamScan";
pub const MOONBEAM_SUBMIT_LOG_TARGET: &str = "MoonbeamSubmit";
pub const MOONBEAM_QUERY_LOG_TARGET: &str = "MoonbeamQuery";

// contract function which keeper use to submit verification result
pub const SUBMIT_VERIFICATION: &str = "submit";
pub const SUBMIT_STATUS_QUERY: &str = "hasSubmitted";
pub const IS_FINISHED: &str = "isFinished";


#[derive(Debug, Clone)]
pub struct Service {
    pub config: MoonbeamConfig,
    pub metrics: Option<MoonbeamMetrics>,
    pub registry: Option<Registry>,
    // client that handle connections
    pub client: MoonbeamClient,
    pub keeper_setting: KeeperSetting,
    // todo: verify publickey derived from prk equals keeper address
    private_key: SecretKey
}

impl Service {

    pub fn private_key(&self) -> SecretKey {
        self.private_key
    }
}

#[derive(Default)]
pub struct ServiceBuilder {
    config: MoonbeamConfig,
    metrics: Option<MoonbeamMetrics>,
    registry: Option<Registry>,
    keeper_setting: KeeperSetting
}

impl ServiceBuilder {

    // must initialize ServiceBuilder with a config
    pub fn new(config: MoonbeamConfig) -> Self {
        Self { config, ..Default::default() }
    }

    // inject prometheus metrics
    pub fn inject_metrics(mut self, metrics: MoonbeamMetrics, registry: Registry) -> Self {
        self.metrics = Some(metrics);
        self
    }

    // inject prometheus registry
    pub fn inject_registry(mut self, registry: Registry) -> Self {
        self.registry = Some(registry);
        self
    }

    pub fn inject_keeper_setting(mut self, keeper: KeeperSetting) -> Self {
        self.keeper_setting = keeper;
        self
    }

    pub fn build(self) -> Result<Service, Error> {
        // get secretkey from config
        let private_key = SecretKey::from_str(&self.config.private_key).map_err(|e| Error::PrivateKeyError(e))?;

        // client
        let client = MoonbeamClient::new(&self.config.url)?;
        
        Ok(
            Service {
                config: self.config,
                metrics: self.metrics,
                registry: self.registry,
                client,
                keeper_setting: self.keeper_setting,
                private_key
            }
        )
    }
}



#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
#[derive(Default)]
pub struct MoonbeamConfig {
    pub url: String,
    // where users add their proofs and emit `AddProof` event
    pub read_contract: String,
    // where keeper submit the verify result
    pub write_contract: String,
    pub private_key: String,
}

#[derive(Clone, Debug)]
pub struct MoonbeamClient {
    inner: Web3<Http>,
    ip_address: String,
}

impl IpAddress for MoonbeamClient {
    fn ip_address(&self) -> String {
        self.ip_address.to_owned()
    }
}

impl MoonbeamClient {
    pub fn new(url: &str) -> Result<Self, Error> {
        if url.starts_with("http") {
            let web3 = Web3::new(Http::new(&url)?);
            Ok(MoonbeamClient { inner: web3, ip_address: url.to_owned() })
        } else {
            Err(Error::ClientCreationError("Wrong Moonbeam connection url".to_owned()))
        }
    }

    pub fn eth(&self) -> Eth<Http> {
        self.inner.eth()
    }

    pub async fn best_number(&self) -> Result<U64, Error> {
        let maybe_best = self.eth().block_number().await;
        maybe_best.map_err(|e| e.into())
    }

    // get proof contract
    pub fn proof_contract(&self, contract_addr: &str) -> Contract<Http> {
        let address = super::utils::trim_address_str(contract_addr).expect("wrong proof contract address");
        let contract = Contract::from_json(
            self.inner.eth(),
            address,
            include_bytes!("../../primitives/contracts/ProofStorage.json"),
        ).expect("Panic at loading ProofStorage.json");
        contract
    }

    // get submit verification contract
    pub fn aggregator_contract(&self, contract_addr: &str) -> Contract<Http> {
        let address = super::utils::trim_address_str(contract_addr).expect("wrong aggregator contract address");
        let contract = Contract::from_json(
            self.inner.eth(),
            address,
            include_bytes!("../../primitives/contracts/SimpleAggregator.json"),
        ).expect("Panic at loading SimpleAggregator.json");
        contract
    }

    #[cfg(test)]
    pub fn events_contract(&self, contract_addr: &str) -> Result<Contract<Http>, Error> {
        let address = super::utils::trim_address_str(contract_addr)?;
        let contract = Contract::from_json(
            self.inner.eth(),
            address,
            include_bytes!("../contracts/TestDynamicEvent.json"),
        )?;
        Ok(contract)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_cargo_env_variables() {
        let _contract_name = "KiltProofs";
        let bytes = include_bytes!("../contracts/ProofStorage.json");
        assert!(bytes.len() != 0);
    }
}
