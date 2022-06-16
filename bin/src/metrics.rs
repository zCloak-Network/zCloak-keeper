use lazy_static::lazy_static;

use prometheus_endpoint::{Counter, Gauge, PrometheusError, Registry, U64};

lazy_static! {
	pub static ref TOKIO_THREADS_TOTAL: Counter<U64> =
		Counter::new("zcloak_keeper_tokio_threads_total", "Total number of threads created")
			.expect("Creating of statics doesn't fail. qed");
}

pub fn register_globals(registry: &Registry) -> Result<(), PrometheusError> {
	registry.register(Box::new(TOKIO_THREADS_TOTAL.clone()))?;
	Ok(())
}
