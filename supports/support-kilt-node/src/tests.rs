pub use super::*;
use hex;
use hex::FromHex;

#[test]
fn storage_key_should_be_correct() {
	let root_hash = Hash::from_slice(
		&hex::decode("ed263f14fe2477486ebb59aaaec0c4cf1e2455ef6f3bda24c08700139ad59ce0").unwrap(),
	);
	let key = [99, 16, 254, 212, 115, 25, 182, 88, 249, 184, 178, 80, 78, 13, 114, 236, 174, 57, 77, 135,
		157, 223, 127, 153, 89, 91, 192, 221, 54, 227, 85, 181, 242, 3, 7, 104, 178, 243, 157, 195,
		229, 20, 124, 133, 217, 129, 185, 123, 237, 38, 63, 20, 254, 36, 119, 72, 110, 187, 89,
		170, 174, 192, 196, 207, 30, 36, 85, 239, 111, 59, 218, 36, 192, 135, 0, 19, 154, 213, 156,
		224];
	let storage_key = get_attestation_storage_key::<Hash>(root_hash);
	assert_eq!(storage_key, StorageKey(key.to_vec()));
}

#[tokio::test]
async fn storage_should_return() {
	let peregrine_url = "wss://full-nodes.kilt.io:9944";
	let total_supply_key = get_storage_value_key("Balances", "TotalIssuance");

	let kilt_client = KiltClient::try_from_url(peregrine_url).await.unwrap();

	let maybe_total_supply = kilt_client.storage(&total_supply_key, None).await;
	let supply: u128 =
		Decode::decode(&mut maybe_total_supply.unwrap().unwrap().0.as_slice()).unwrap();
	assert!(supply > 45 * 10_u128.pow(26) as u128);
}

#[tokio::test]
async fn should_return_value() {
	let peregrine_url = "wss://peregrine.kilt.io:443/";
	let kilt_client = KiltClient::try_from_url(peregrine_url).await.unwrap();
	let root_hash = Hash::from_slice(
		&hex::decode("ed263f14fe2477486ebb59aaaec0c4cf1e2455ef6f3bda24c08700139ad59ce0").unwrap(),
	);
	let storage_key = get_attestation_storage_key::<Hash>(root_hash);
	let maybe_maybe_attestation = kilt_client.storage(&storage_key, None).await.unwrap();

	let expected_raw_storage = "7f2ef721b292b9b7d678e9f82ab010e139600558df805bbc61a0041e60b61a18cc1220acdcc086a650a89b140349ef064459718eaff3896a6e45dd9664211d810000a226c84c99568a074d98c71f55131f0512c12e9ecf9de0d91b30bcc856844b140080c6a47e8d03000000000000000000";
	assert_eq!(expected_raw_storage, hex::encode(maybe_maybe_attestation.unwrap().0));
}


#[test]
fn storage_decode_should_work() {
	let mut expected_raw_storage = "7f2ef721b292b9b7d678e9f82ab010e139600558df805bbc61a0041e60b61a18cc1220acdcc086a650a89b140349ef064459718eaff3896a6e45dd9664211d810000a226c84c99568a074d98c71f55131f0512c12e9ecf9de0d91b30bcc856844b140080c6a47e8d03000000000000000000";
	let attestation: Attestation = Decode::decode(&mut hex::decode(expected_raw_storage).unwrap().as_slice()).unwrap();
	assert!(!attestation.revoked);
}


#[tokio::test]
async fn should_get_right_attestation() {
	let peregrine_url = "wss://peregrine.kilt.io:443/";
	let hex_key =
		hex::decode("ed263f14fe2477486ebb59aaaec0c4cf1e2455ef6f3bda24c08700139ad59ce0").unwrap();
	let root_hash = Hash::from_slice(&hex_key);
	let storage_key = get_attestation_storage_key::<Hash>(root_hash);
	let kilt_client = KiltClient::try_from_url(peregrine_url).await.unwrap();

	let maybe_attestation = kilt_client.storage(&storage_key, None).await.unwrap();
	let attestation: Attestation =
		Decode::decode(&mut maybe_attestation.unwrap().0.as_slice()).unwrap();

	assert_eq!(
		attestation.ctype_hash,
		Hash::from_slice(
			&hex::decode("7f2ef721b292b9b7d678e9f82ab010e139600558df805bbc61a0041e60b61a18")
				.unwrap()
		)
	);
	assert!(!attestation.revoked);
}


#[tokio::test]
async fn query_attestation_should_work() {
	let peregrine_url = "wss://peregrine.kilt.io:443/";
	let hex_key =
		hex::decode("ed263f14fe2477486ebb59aaaec0c4cf1e2455ef6f3bda24c08700139ad59ce0").unwrap();
	let root_hash = Hash::from_slice(&hex_key);

	let res = query_attestation(peregrine_url, root_hash).await.unwrap();
	assert!(res);

}