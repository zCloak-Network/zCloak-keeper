use codec::Encode;
use super::*;
use sp_core::{
    Bytes,
    H256 as Hash, storage::{StorageData, StorageKey},
};

use super::types::{ATTESTATION_PALLET_PREFIX, ATTESTATION_STORAGE_PREFIX, HASHER};
use frame_metadata::StorageHasher;

/// get the storage key of attestations
pub fn get_attestation_storage_key<Key: Encode>(key: Key) -> StorageKey {
    get_storage_map_key(key, ATTESTATION_PALLET_PREFIX, ATTESTATION_STORAGE_PREFIX)
}

fn get_storage_map_key<Key: Encode>(
    key: Key,
    pallet_prefix: &str,
    storage_prefix: &str,
) -> StorageKey {
    let mut bytes = sp_core::twox_128(pallet_prefix.as_bytes()).to_vec();
    bytes.extend(&sp_core::twox_128(storage_prefix.as_bytes())[..]);
    bytes.extend(key_hash(&key, &HASHER));
    StorageKey(bytes)
}

#[allow(unused)]
fn get_storage_value_key(pallet_prefix: &str, storage_prefix: &str) -> StorageKey {
    let mut bytes = sp_core::twox_128(pallet_prefix.as_bytes()).to_vec();
    bytes.extend(&sp_core::twox_128(storage_prefix.as_bytes())[..]);
    StorageKey(bytes)
}

fn key_hash<K: Encode>(key: &K, hasher: &StorageHasher) -> Vec<u8> {
    let encoded_key = key.encode();
    match hasher {
        StorageHasher::Identity => encoded_key.to_vec(),
        StorageHasher::Blake2_128 => sp_core::blake2_128(&encoded_key).to_vec(),
        StorageHasher::Blake2_128Concat => {
            // copied from substrate Blake2_128Concat::hash since StorageHasher is not public
            let x: &[u8] = encoded_key.as_slice();
            sp_core::blake2_128(x).iter().chain(x.iter()).cloned().collect::<Vec<_>>()
        },
        StorageHasher::Blake2_256 => sp_core::blake2_256(&encoded_key).to_vec(),
        StorageHasher::Twox128 => sp_core::twox_128(&encoded_key).to_vec(),
        StorageHasher::Twox256 => sp_core::twox_256(&encoded_key).to_vec(),
        StorageHasher::Twox64Concat =>
            sp_core::twox_64(&encoded_key).iter().chain(&encoded_key).cloned().collect(),
    }
}