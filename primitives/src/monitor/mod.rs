pub use notify_bot::*;
pub use prometheus::{
	core::{
		AtomicF64 as F64, AtomicI64 as I64, AtomicU64 as U64, GenericCounter as Counter,
		GenericCounterVec as CounterVec, GenericGauge as Gauge, GenericGaugeVec as GaugeVec,
	},
	Error as PrometheusError, Registry as PrometheusRegistry,
};

pub mod notify_bot;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("POST monitor bot error, reason: {0}")]
	HttpError(#[from] reqwest::Error),
	#[error("Monitor message pack error, err: {0}")]
	TemplateFormatError(#[from] strfmt::FmtError),
}

pub type Result<T> = std::result::Result<T, Error>;
