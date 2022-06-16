pub use notify_bot::*;

pub mod notify_bot;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("POST monitor bot error, reason: {0}")]
	HttpError(#[from] reqwest::Error),
	#[error("Monitor message pack error, err: {0}")]
	TemplateFormatError(#[from] strfmt::FmtError),
}

pub type Result<T> = std::result::Result<T, Error>;
