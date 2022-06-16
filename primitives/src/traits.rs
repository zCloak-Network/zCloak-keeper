// pub trait JsonParse {
//
//     fn into_bytes(self) -> serde_json::Result<Vec<u8>>;
//
//     fn try_from_bytes(json: &[u8]) -> serde_json::Result<Self> where Self: Sized;
// }

use std::fmt::Debug;

pub trait JsonParse<E> {
	fn into_bytes(self) -> Result<Vec<u8>, E>;

	fn try_from_bytes(json: &[u8]) -> Result<Self, E>
	where
		Self: Sized;
}


pub trait IpAddress {
	fn ip_address(&self) -> String;
}

pub trait IntoStr {
	fn into_str(&self) -> String;
}

impl<T: Debug> IntoStr for T {
	fn into_str(&self) -> String {
		format!("{:?}", &self)
	}
}