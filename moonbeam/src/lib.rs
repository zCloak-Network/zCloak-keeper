
use secp256k1::SecretKey;
use std::{collections::BTreeMap, str::FromStr};
use web3::{signing::SecretKeyRef, Transport, types::U64};
use web3::{
    contract::{Contract, tokens::Detokenize, Options},
    signing::Key,
    transports::Http,
    Web3
};
use web3::types::{Address, BlockNumber, FilterBuilder, Log};
use keeper_primitives::{IpfsClient, IpfsConfig, MoonbeamClient};
use keeper_primitives::{
    VerifyResult,
    Result as KeeperResult,
    moonbeam::{self,
        MOONBEAM_BLOCK_DURATION,
        MOONBEAM_TRANSACTION_CONFIRMATIONS,
        MOONBEAM_LISTENED_EVENT,
        MOONBEAM_SCAN_SPAN,
        ProofEvent,
        MoonbeamConfig,
    }
};

// TODO: before get into scan_events
// record the scanned block number somewhere(e.g. file)
// compare the file-recorded block to the one passed from the
// command the choose the latest one.

// initialize connection and contract before scan_event
pub async fn scan_events<T: Transport>(
    mut start: U64,
    web3: &Web3<T>,
    proof_contract: &Contract<T>,
) -> KeeperResult<BTreeMap<U64, Vec<ProofEvent>>> {
    let maybe_best = web3.eth().block_number().await;
    let best = match maybe_best {
        Ok(b) => b,
        Err(e) => return Err((U64::default(), e.into()))
    };
    // if start > finalized, reset `start` pointer to best
    if start > best {
        log::warn!("scan moonbeam start block is higher than current best! start_block={}, best_block:{}", start, best);
        start = best;
    }
    let span: U64 = MOONBEAM_SCAN_SPAN.into();
    let end = if start + span > best { best } else { start + span };

    log::info!(
			"try to scan moonbeam log from block [{:}] - [{:}] | best:{:}",
			start,
			end,
			best
    );
    // parse event
    let r = moonbeam::utils::events::<_, ProofEvent>(
        web3.eth(),
        proof_contract,
        MOONBEAM_LISTENED_EVENT,
        Some(start),
        Some(end),
    ).await;

    // if event parse error, return Err(start) and output error log
    let r = match r {
        Ok(events) => events,
        Err(err) => {
            log::error!("Moonbeam Scan Err: Event parse error. {:?}", err);
            return Err((start, err.into()));
        }
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
    Ok(result)
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
                (v.data_owner, worker_address, v.root_hash, v.c_type, v.program_hash),
                None,
                Options::default(),
                None,
            )
            .await?;
        log::info!("[Moonbeam] hasSubmitted result is {}", r);
        if !r {
            let r = contract
                .signed_call_with_confirmations(
                    "addVerification",
                    (v.data_owner, v.root_hash, v.c_type, v.program_hash, v.is_passed),
                    {
                        let mut options = Options::default();
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

