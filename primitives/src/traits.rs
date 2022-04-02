pub trait JsonParse {

    fn into_json_str(self) -> serde_json::Result<Vec<u8>>;

    fn from_json_str(json: &[u8]) -> Self;
}
