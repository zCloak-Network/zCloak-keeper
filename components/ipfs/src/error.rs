#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Just allow specified ipfs host")]
	InvalidIpfsHost,
	#[error("Request IPFS error, reason: {0}")]
	HttpError(#[from] reqwest::Error),
	#[error("Assembly Url error, reason: {0}")]
	UrlError(#[from] url::ParseError),
	#[error("Set Scheme Error")]
	SchemeError,
}
pub type Result<T> = std::result::Result<T, Error>;
