use keeper_primitives::{
	moonbeam::{
		self, ProofEvent, MOONBEAM_LISTENED_EVENT, MOONBEAM_SCAN_SPAN,
		MOONBEAM_TRANSACTION_CONFIRMATIONS,
	},
	Contract, Http, MoonbeamClient, Result as KeeperResult, VerifyResult, Web3Options, U64,
};
use secp256k1::SecretKey;
use std::collections::BTreeMap;
use web3::signing::{Key, SecretKeyRef};

// TODO: before get into scan_events
// record the scanned block number somewhere(e.g. file)
// compare the file-recorded block to the one passed from the
// command the choose the latest one.

// initialize connection and contract before scan_event

pub async fn scan_events(
	mut start: U64,
	client: &MoonbeamClient,
	proof_contract: &Contract<Http>,
) -> KeeperResult<(BTreeMap<U64, Vec<ProofEvent>>, U64)> {
	let maybe_best = client.best_number().await;
	let best = match maybe_best {
		Ok(b) => b,
		Err(e) => return Err((U64::default(), e.into())),
	};
	// if start > best, reset `start` pointer to best
	if start > best {
		log::warn!(
			"scan moonbeam start block is higher than current best! start_block={}, best_block:{}",
			start,
			best
		);
		start = best;
	}
	let span: U64 = MOONBEAM_SCAN_SPAN.into();
	let end = if start + span > best { best } else { start + span };

	log::info!("try to scan moonbeam log from block [{:}] - [{:}] | best:{:}", start, end, best);
	// parse event
	let r = moonbeam::utils::events::<_, ProofEvent>(
		client.eth(),
		proof_contract,
		MOONBEAM_LISTENED_EVENT,
		Some(start),
		Some(end),
	)
	.await;

	// if event parse error, return Err(start) and output error log
	let r = match r {
		Ok(events) => events,
		Err(err) => {
			log::error!("Moonbeam Scan Err: Event parse error. {:?}", err);
			return Err((start, err.into()))
		},
	};

	let hit = r.len();

	let mut result = BTreeMap::<U64, Vec<ProofEvent>>::default();
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

pub async fn submit_tx(
	contract: &Contract<Http>,
	worker: SecretKey,
	res: Vec<VerifyResult>,
) -> std::result::Result<(), keeper_primitives::moonbeam::Error> {
	log::info!("[Moonbeam] submiting the tx");
	let key_ref = SecretKeyRef::new(&worker);
	let worker_address = key_ref.address();
	for v in res {
		// TODO: read multiple times?
		let r: bool = contract
			.query(
				"hasSubmitted",
				(worker_address, v.request_hash),
				None,
				Web3Options::default(),
				None,
			)
			.await?;
		log::info!("[Moonbeam] hasSubmitted result is {}", r);

		if !r {
			let r = contract
				.signed_call_with_confirmations(
					"submit",
					(v.data_owner, v.request_hash, v.c_type, v.root_hash, v.is_passed, v.attester),
					{
						let mut options = Web3Options::default();
						options.gas = Some(1000000.into());
						options
					},
					MOONBEAM_TRANSACTION_CONFIRMATIONS,
					&worker,
				)
				.await?;
			// TODO handle result for some error
			log::warn!("[Moonbeam] receipt is {:?}", r);
			log::info!(
				"[moonbeam] submit verification|tx:{:}|data owner:{:}|root_hash:{:}",
				r.transaction_hash,
				v.data_owner,
				hex::encode(v.root_hash)
			);
		}
	}

	Ok(())
}
