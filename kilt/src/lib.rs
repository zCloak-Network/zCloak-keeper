use keeper_primitives::{VerifyResult, U64, Hash, StorageKey, Encode, Decode};
use keeper_primitives::{
    kilt::{KiltClient, Attestation, KILT_MAX_RETRY_TIMES, Error, get_attestation_storage_key},
};
use jsonrpsee::types::Error as RpcError;

pub async fn filter(
    url: &str,
    result: Vec<VerifyResult>,
) -> std::result::Result<Vec<VerifyResult>, (U64, Error)> {
    let mut v = vec![];
    for i in result {
        let r = query_attestation(url, i.root_hash.into()).await.map_err(|e| (i.number, e.into()))?;
        if r {
            v.push(i)
        } else {
            log::error!("[kilt] attestaion is not valid for this root_hash|root_hash:{:}|data owner:{:}|number:{:}", hex::encode(i.root_hash), hex::encode(i.data_owner.0), i.number);
        }
    }
    Ok(v)
}


/// query attestation info from kilt network
/// TODO: handle kilt error??
pub async fn query_attestation(url: &str, root_hash: Hash) -> std::result::Result<bool, Error> {
    let client = KiltClient::try_from_url(url).await?;
    let storage_key = get_attestation_storage_key::<Hash>(root_hash);
    let mut times = 0;
    let maybe_attestation_details = loop {
        // connect to kilt and query attestation storage
        // TODO: Attestaion or Option<Attestation>
        match client.storage(&storage_key, None).await {
            Ok(details) => break details,
            Err(e) => {
                match e {
                    RpcError::RequestTimeout | RpcError::Transport(_) | RpcError::Request(_) =>
                        if times < KILT_MAX_RETRY_TIMES {
                            times += 1;
                            log::warn!(
								"query kilt storage timeout, retry {:}/{:}",
								times,
								KILT_MAX_RETRY_TIMES
							);
                            continue
                        },

                    _ => {},
                }
                return Err(e)?
            },
        }
    };

    // decode fetched storage data
    let is_valid = match maybe_attestation_details {
        Some(mut data) => {
            let attestation: Attestation = Decode::decode(&mut data.0.as_slice())?;
            // valid if the attestation record is not revoked by the kyc agent
            !attestation.revoked
        },
        None => false,
    };
    println!("VALID is {}", is_valid);
    Ok(is_valid)
}
