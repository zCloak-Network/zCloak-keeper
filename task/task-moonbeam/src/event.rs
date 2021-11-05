use web3::{
    ethabi::LogParam,
    types::Address,
    types::U128,
};
use std::convert::TryInto;
use std::convert::TryFrom;



#[derive(Debug, Default)]
pub struct CreateTaskEvent {
    pub sender: Address,
    pub program_hash: [u8; 32],
    pub public_inputs: Vec<u128>,
    pub outputs: Vec<u128>,
    pub proof_id: Vec<u8>,
    pub program: Vec<u8>,
}

impl CreateTaskEvent {

    pub fn parse_log(params: Vec<LogParam>) -> anyhow::Result<CreateTaskEvent> {

        let mut create_param = CreateTaskEvent::default();
        for param in params {
            match param.name.as_str() {
                "sender" => {
                    create_param.sender = param.value.into_address().unwrap();
                }
                "programHash" => {
                    let bytes = param.value.into_bytes().unwrap();
                    let buf = pop(&bytes[..]);
                    create_param.program_hash = buf;
                    create_param.program = bytes.clone();
                }
                "publicInputs" => {
                    let arrays = param.value.into_array().unwrap();
                    for array in arrays {
                        let value = array.into_uint().unwrap();
                        let vv = U128::try_from(value).unwrap();
                        create_param.public_inputs.push(vv.low_u128());
                    }
                }
                "outputs" => {
                    let arrays = param.value.into_array().unwrap();
                    for array in arrays {
                        let value = array.into_uint().unwrap();
                        let vv = U128::try_from(value).unwrap();
                        create_param.outputs.push(vv.low_u128());
                    }
                }
                "proofId" => {
                    let proof_id = param.value.into_string().unwrap();
                    create_param.proof_id = proof_id.as_bytes().to_vec();
    
                }
                _ => {
    
                }
            }
        }
        log::info!("create param is {:?}", create_param);
        return Ok(create_param);
    
    }
}



fn pop(barry: &[u8]) -> [u8; 32] {
    barry.try_into().expect("slice with incorrect length")
}