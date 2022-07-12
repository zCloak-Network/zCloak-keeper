pub trait JsonParse {
	fn into_bytes(self) -> std::result::Result<Vec<u8>, super::error::Error>;

	fn try_from_bytes(json: &[u8]) -> std::result::Result<Self, super::error::Error>
	where
		Self: Sized;
}
