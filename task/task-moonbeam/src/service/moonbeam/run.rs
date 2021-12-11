use crate::{
	config::{ContractConfig, KiltConfig, MoonbeamConfig},
	error::{Error, Result},
};
use component_ipfs::IpfsConfig;
use web3::types::U64;

pub async fn run_worker(
	start_number: Option<u64>,
	moonbeam: MoonbeamConfig,
	contract: ContractConfig,
	ipfs: IpfsConfig,
	kilt: KiltConfig,
) -> Result<()> {
	let web3 = scan_moonbeam::web3eth(&moonbeam)?;
	let proof_contract = scan_moonbeam::kilt_proofs_contract(&web3, &contract)?;

	// if user not set start_number, then use best number as the start number
	let mut start = if let Some(s) = start_number.map(|n| n.into()) {
		s
	} else {
		web3.eth().block_number().await?
	};

	loop {
		let (res, end) = scan_moonbeam::scan_events(start, &web3, &proof_contract).await?;
		start = end;
		if res.is_empty() {
			if start == web3.eth().block_number().await? {
				// if current start is the best number, then sleep the block duration.
				use tokio::time::{sleep, Duration};
				sleep(Duration::from_secs(scan_moonbeam::MOONBEAM_BLOCK_DURATION)).await;
			}
			continue
		}
		// handle result
	}
	Ok(())
}

mod scan_moonbeam {
	use super::*;
	use log::info;
	use starksVM::OpCode::Add;
	use std::collections::BTreeMap;
	use web3::{
		api::Eth,
		contract::{
			tokens::{Detokenize, Tokenize},
			Contract, Error as Web3ContractErr,
		},
		ethabi,
		transports::Http,
		types::{Address, BlockNumber, FilterBuilder, Log, U256, U64},
		Error as Web3Err, Transport, Web3,
	};

	// TODO can set this scan span block number as a config in future
	const SCAN_SPAN: usize = 10;
	const LISTENED_EVENT: &'static str = "AddProof";
	pub const MOONBEAM_BLOCK_DURATION: u64 = 12;

	pub type Bytes32 = [u8; 32];

	#[derive(Debug, Default)]
	pub struct Proof {
		data_owner: Address,
		kilt_address: Bytes32,
		c_type: Bytes32,
		program_hash: Bytes32,
		field_name: String,
		proof_cid: String,
		root_hash: Bytes32,
		expect_result: bool,
	}
	impl From<ProofEventType> for Proof {
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

	type ProofEventType = (Address, Bytes32, Bytes32, Bytes32, String, String, Bytes32, bool);

	pub fn web3eth(config: &MoonbeamConfig) -> std::result::Result<Web3<Http>, Web3Err> {
		let http = web3::transports::Http::new(&config.url)?;
		Ok(web3::Web3::new(http))
	}

	pub fn kilt_proofs_contract(
		web3: &Web3<Http>,
		config: &ContractConfig,
	) -> Result<Contract<Http>> {
		let addr =
			if config.address.starts_with("0x") { &config.address[2..] } else { &config.address };
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
			web3.eth(),
			address,
			include_bytes!("../../../contracts/KiltProofs.json"),
		)?;
		Ok(kilt_proofs_v1)
	}

	pub async fn scan_events(
		mut start: U64,
		web3: &Web3<Http>,
		contract: &Contract<Http>,
	) -> Result<(BTreeMap<U64, Vec<Proof>>, U64)> {
		let best = web3.eth().block_number().await?;
		if start > best {
			log::warn!("scan moonbeam start block is higher than current best! start_block={}, best_block:{}", start, best);
			start = best;
		}
		let span: U64 = SCAN_SPAN.into();
		let end = if start + span > best { best } else { start + span };

		log::info!("try to can moonbeam log from block [{:}] - [{:}] | best:{:}", start, end, best);
		let r = events::<_, ProofEventType>(
			web3.eth(),
			contract,
			LISTENED_EVENT,
			Some(start),
			Some(end),
		)
		.await?;

		let hit = r.len();

		let mut result = BTreeMap::<U64, Vec<Proof>>::default();
		for (proof_event, log) in r {
			let number = log.block_number.unwrap_or_else(|| {
				log::warn!("Moonbeam log blocknumber should not be None");
				Default::default()
			});
			result.entry(number).or_insert(vec![]).push(proof_event.into());
		}
		log::info!(
			"scan from [{:}] - [{:}] | hit:[{:}] | in blocks: {:?}",
			start,
			end,
			hit,
			result.keys().into_iter().map(|n| *n).collect::<Vec<U64>>()
		);
		Ok((result, end))
	}

	async fn events<T: Transport, R: Detokenize>(
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
