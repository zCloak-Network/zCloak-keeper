// todo: consider separate the metrics into independent ones

use keeper_primitives::monitor::{register, Counter, PrometheusError, PrometheusRegistry, U64};
use std::sync::Arc;

pub struct MoonbeamMetrics {
	pub submitted_verify_transactions: Counter<U64>,
}

impl MoonbeamMetrics {
	pub fn register(registry: &PrometheusRegistry) -> std::result::Result<Self, PrometheusError> {
		Ok(Self {
			submitted_verify_transactions: register(
				Counter::new(
					"keeper_submitted_veirify_transactions",
					"Total number of [verify proof] transactions that a keeper has submitted",
				)?,
				registry,
			)?,
		})
	}
}

/// An extension trait for [`BasicMetrics`].
pub trait MoonbeamMetricsExt {
	/// Report an event to the metrics.
	fn report(&self, report: impl FnOnce(&MoonbeamMetrics));
}

impl MoonbeamMetricsExt for Option<Arc<MoonbeamMetrics>> {
	fn report(&self, report_fn: impl FnOnce(&MoonbeamMetrics)) {
		if let Some(metrics) = self.as_ref() {
			report_fn(metrics)
		}
	}
}
