
use crate::utils::ipfs::client::IpfsClient;

use starksVM as stark;
use std::str;




pub async fn verifier_proof(
    task_name: String,
    ipfs_client: &IpfsClient,
    proof_id: Vec<u8>, 
    program_hash: [u8; 32], 
    public_inputs: Vec<u128>, 
    outputs: Vec<u128>) -> anyhow::Result<bool, anyhow::Error> {
        let body = ipfs_client.fetch_proof(&proof_id).await?;
        let proof = hex::decode(&body[0..body.len()]);
        let mut res = false;
        match proof {
            Ok(proof) => {
                let stark_proof = bincode::deserialize::<stark::StarkProof>(&proof);
                    match stark_proof {
                        Ok(stark_proof) => {
                            let is_success =
                            stark::verify(&program_hash, &public_inputs, &outputs, &stark_proof);
                            res = if let Ok(r) = is_success {
                                log::debug!(
                                    "task name:{:?} , proofid {:?} stark verify true !",&task_name,
                                    str::from_utf8(&proof_id).unwrap()
                                );
                                r
                            } else {
                                log::debug!(
                                    "task name:{:?} , proofid {:?} stark verify false !",&task_name,
                                    str::from_utf8(&proof_id).unwrap()
                                );
                                false
                            };
                        },
                        Err(e) => {
                            log::error!("task name:{:?} service , bincode deserialize got failed, exception stack is {:?}",&task_name, e);
                        }
                    }
            },
            Err(e) => {
                log::error!("task name:{:?} , proof id:{:?} , hex decode the content failed which 
                got from ipfs!, error stack is {:?}", &task_name, str::from_utf8(&proof_id).unwrap(),e);
            }
        }
        return Ok(res)
}