use web3::{
	self as web3,
	api::Eth,
	contract::{
		tokens::{Detokenize, Tokenize},
		Contract, Error as Web3ContractErr,
	},
	ethabi,
	transports::Http,
	Transport,
};

pub use super::*;
use super::{Deserialize, Serialize};

pub const MOONBEAM_SCAN_SPAN: usize = 10;
// TODO: move it to config file
pub const MOONBEAM_LISTENED_EVENT: &'static str = "AddProof";
pub const MOONBEAM_BLOCK_DURATION: u64 = 12;
pub const MOONBEAM_TRANSACTION_CONFIRMATIONS: usize = 1;
pub const MOONBEAM_SCAN_LOG_TARGET: &str = "MoonbeamScan";
pub const MOONBEAM_SUBMIT_LOG_TARGET: &str = "MoonbeamSubmit";
// contract function which keeper use to submit verification result
pub const SUBMIT_VERIFICATION: &str = "submit";
pub const SUBMIT_STATUS_QUERY: &str = "hasSubmitted";
pub const IS_FINISHED: &str = "isFinished";
pub const QUEUE_EXPIRE_DURATION: u8 = 30;

#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
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
}

impl MoonbeamClient {
	pub fn new(url: String) -> Result<Self> {
		if url.starts_with("http") {
			let web3 = Web3::new(Http::new(&url)?);
			Ok(MoonbeamClient { inner: web3 })
		} else {
			Err(Error::ClientCreationError("Wrong Moonbeam connection url".to_owned()))
		}
	}

	pub fn eth(&self) -> Eth<Http> {
		self.inner.eth()
	}

	pub async fn best_number(&self) -> Result<U64> {
		let maybe_best = self.eth().block_number().await;
		maybe_best.map_err(|e| e.into())
	}

	// get proof contract
	pub fn proof_contract(&self, contract_addr: &str) -> Result<Contract<Http>> {
		let address = utils::trim_address_str(contract_addr)?;
		let contract = Contract::from_json(
			self.inner.eth(),
			address,
			include_bytes!("../contracts/ProofStorage.json"),
		)?;
		Ok(contract)
	}

	// get submit verification contract
	pub fn aggregator_contract(&self, contract_addr: &str) -> Result<Contract<Http>> {
		let address = utils::trim_address_str(contract_addr)?;
		let contract = Contract::from_json(
			self.inner.eth(),
			address,
			include_bytes!("../contracts/SimpleAggregator.json"),
		)?;
		Ok(contract)
	}

	#[cfg(test)]
	pub fn events_contract(&self, contract_addr: &str) -> Result<Contract<Http>> {
		let address = utils::trim_address_str(contract_addr)?;
		let contract = Contract::from_json(
			self.inner.eth(),
			address,
			include_bytes!("../contracts/TestDynamicEvent.json"),
		)?;
		Ok(contract)
	}
}

pub mod utils {
	use super::*;

	// todo: test if if can filter event due to contract address
	pub async fn events<T: Transport, R: Detokenize>(
		web3: Eth<T>,
		contract: &Contract<T>,
		event: &str,
		from: Option<U64>,
		to: Option<U64>,
	) -> Result<Vec<(R, Log)>> {
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

		// filter event by address
		builder = builder.address(vec![contract.address()]);

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

	pub(super) fn trim_address_str(addr: &str) -> Result<Address> {
		let addr = if addr.starts_with("0x") { &addr[2..] } else { addr };
		let hex_res =
			hex::decode(addr).map_err(|e| Error::InvalidEthereumAddress(format!("{:}", e)))?;
		// check length
		if hex_res.len() != 20 {
			return Err(Error::InvalidEthereumAddress(format!(
				"Address is not equal to 20 bytes: {:}",
				addr
			)))
		}
		let address = Address::from_slice(&hex_res);
		Ok(address)
	}
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Moonbeam connection Error: {0}")]
	ClientCreationError(String),
	#[error("Web3 Client Error, err: {0}")]
	Web3Error(#[from] web3::Error),

	#[error("Web3 Contract Error, err: {0}")]
	Web3ContractError(#[from] Web3ContractErr),

	#[error("Ethereum Abi Error, err: {0}")]
	EthAbiError(#[from] web3::ethabi::Error),

	#[error("Invalid Ethereum Address: {0}")]
	InvalidEthereumAddress(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
	#[test]
	fn test_cargo_env_variables() {
		let _contract_name = "KiltProofs";
		let bytes = include_bytes!("../contracts/ProofStorage.json");
		assert!(bytes.len() != 0);
	}

	// #[tokio::test]
	// async fn dynamic_array_in_event_should_parse_right() {
	// 	let mock_client = MoonbeamClient::new("http://127.0.0.1:7545".to_owned())
	// 		.expect("moonbeam client url is wrong");
	// 	let test_contract = mock_client
	// 		.events_contract("0xb364A9B9bE6E1d66A41b8a4AA15F5311968EB44C")
	// 		.expect("contract should be deployed");
	// 	type EventEnum = (Address, Vec<u128>);
	// 	let res = utils::events::<_, EventEnum>(
	// 		mock_client.eth(),
	// 		&test_contract,
	// 		"Dynamic",
	// 		Some(204.into()),
	// 		Some(204.into()),
	// 	)
	// 	.await
	// 	.expect("Wrong log");
	//
	// 	for (event, log) in res {
	// 		assert_eq!(
	// 			event.0,
	// 			Address::from_str("69d09ef8b6B1a2fECD70F147bA302B8278cafF39")
	// 				.expect("wrong address format")
	// 		);
	// 		assert_eq!(event.1, vec![1, 2, 3, 4]);
	// 	}
	// }
}
