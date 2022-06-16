use keeper_primitives::traits::IntoStr;

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

    #[error("Events parse Error, error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    VMVerifyError(#[from] keeper_primitives::verify::Error),
}