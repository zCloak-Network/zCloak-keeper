// todo: consider separate the metrics into independent ones

use super::*;
use std::sync::Arc;

pub struct BasicMetrics {
	threads_alive: Gauge<U64>,
}

impl BasicMetrics {
	pub fn register(
		registry: &PrometheusRegistry,
	) -> std::result::Result<Self, super::PrometheusError> {
		Ok(Self {
			threads_alive: register(
				Gauge::new("keeper_threads_alive", "Number of threads alive right now")?,
				registry,
			)?,
		})
	}
}

/// An extension trait for [`BasicMetrics`].
pub trait BasicMetricsExt {
	/// Report an event to the metrics.
	fn report(&self, report: impl FnOnce(&BasicMetrics));
}

impl BasicMetricsExt for Option<Arc<BasicMetrics>> {
	fn report(&self, report_fn: impl FnOnce(&BasicMetrics)) {
		if let Some(metrics) = self.as_ref() {
			report_fn(metrics)
		}
	}
}
