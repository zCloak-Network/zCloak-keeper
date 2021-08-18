#![allow(missing_docs)]

use thiserror::Error as ThisError;

pub type BasicResult<T> = Result<T, BasicError>;

#[derive(ThisError, Debug)]
pub enum BasicError {
    #[error("Crypto error: {0}")]
    Crypto(String),
}
