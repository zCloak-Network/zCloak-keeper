use web3::{
    contract::{
        tokens::{Tokenize, Detokenize},
        Options as Web3Options,
        Contract
    },
    types::{FilterBuilder, Address, U64, Log, BlockNumber},
    ethabi,
    Transport,
    api::Eth
};
use keeper_primitives::Bytes32;
use super::{error::Error, types::MOONBEAM_QUERY_LOG_TARGET, MoonbeamResult};

pub async fn query_submit_and_finish_result<
    T: Transport,
    P1: Tokenize + std::marker::Copy,
    P2: Tokenize + std::marker::Copy,
>(
    contract: &Contract<T>,
    func_1: &str,
    params_1: P1,
    func_2: &str,
    params_2: P2,
    request_hash: Bytes32,
    query_times: usize,
) -> Result<(bool, bool), Error> {
    let mut maybe_fun_1_query_result =
        contract.query(func_1, params_1, None, Web3Options::default(), None).await;
    let mut maybe_fun_2_query_result =
        contract.query(func_2, params_2, None, Web3Options::default(), None).await;
    match (maybe_fun_1_query_result, maybe_fun_2_query_result) {
        (Ok(query_fuc_1_result), Ok(query_fun_2_result)) =>
            return Ok((query_fuc_1_result, query_fun_2_result)),
        _ => {
            let mut maybe_fun_1_retry_result =
                contract.query(func_1, params_1, None, Web3Options::default(), None).await;
            let mut maybe_fun_2_retry_result =
                contract.query(func_2, params_2, None, Web3Options::default(), None).await;
            for i in 0..query_times {
                if maybe_fun_1_retry_result.is_ok() && maybe_fun_2_retry_result.is_ok() {
                    return Ok((
                        maybe_fun_1_retry_result.unwrap(),
                        maybe_fun_2_retry_result.unwrap(),
                    ))
                } else {
                    maybe_fun_1_retry_result = contract
                        .query(func_1, params_1, None, Web3Options::default(), None)
                        .await;
                    maybe_fun_2_retry_result = contract
                        .query(func_2, params_2, None, Web3Options::default(), None)
                        .await;
                }
            }
            match (maybe_fun_1_retry_result, maybe_fun_2_retry_result) {
                (Ok(maybe_fun_1_retry_result), Err(maybe_fun_2_retry_result)) => {
                    log::warn!(
							target: MOONBEAM_QUERY_LOG_TARGET,
							"The {:?} query for request hash[{:?}] meets error: [{:?}]",
							func_2,
							hex::encode(request_hash),
							maybe_fun_2_retry_result
						);
                    return Err(maybe_fun_2_retry_result.into())
                },
                (Err(maybe_fun_1_retry_result), Ok(maybe_fun_2_retry_result)) => {
                    log::warn!(
							target: MOONBEAM_QUERY_LOG_TARGET,
							"The {:?} query for request hash[{:?}] meets error: [{:?}]",
							func_1,
							hex::encode(request_hash),
							maybe_fun_1_retry_result
						);
                    return Err(maybe_fun_1_retry_result.into())
                },
                (Err(maybe_fun_1_retry_result), Err(maybe_fun_2_retry_result)) => {
                    log::warn!(
						target: MOONBEAM_QUERY_LOG_TARGET,
						"The {:?} and {:?} query for request hash[{:?}] meets error: [{:?} and {:?}]",
						func_1,
						func_2,
						hex::encode(request_hash),
						maybe_fun_1_retry_result,
						maybe_fun_2_retry_result
					);
                    return Err(maybe_fun_1_retry_result.into())
                },
                (Ok(maybe_fun_1_retry_result), Ok(maybe_fun_2_retry_result)) =>
                    return Ok((maybe_fun_1_retry_result, maybe_fun_2_retry_result)),
            }
        },
    }
}

// todo: test if if can filter event due to contract address
pub async fn events<T: Transport, R: Detokenize>(
    web3: Eth<T>,
    contract: &Contract<T>,
    event: &str,
    from: Option<U64>,
    to: Option<U64>,
) -> Result<Vec<(R, Log)>, Error> {
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

pub(super) fn trim_address_str(addr: &str) -> Result<Address, Error> {
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