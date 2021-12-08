use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
	#[error("paraslog FAILED")]
	ParseLog(String),
}
