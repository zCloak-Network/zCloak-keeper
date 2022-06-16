// prometheus should be open to all keepers
use hyper::{
	http::StatusCode,
	server::Server,
	service::{make_service_fn, service_fn},
	Body, Request, Response,
};
pub use prometheus::{
	self,
	core::{AtomicU64 as U64, Collector, GenericCounter as Counter, GenericGauge as Gauge},
	exponential_buckets, Error as PrometheusError, Histogram, HistogramOpts, HistogramVec, Opts,
	Registry,
};
use prometheus::{Encoder, TextEncoder};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
pub use utils::{init_prometheus, register};

const EXTERNAL_PROMETHEUS_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);

#[derive(Debug, Clone)]
pub struct PrometheusConfig {
	/// Port to use.
	pub socket_addr: SocketAddr,
	// todo: change it to Address
	pub keeper_addr: String,
}

impl PrometheusConfig {
	/// Create a new config using the default registry.
	pub fn new_with_default_registry(port: u16, keeper_addr: String) -> Self {
		// default: external
		// todo: give keeper maintainer a chance to choose local or external mode
		let socket_addr = SocketAddr::new(IpAddr::from(EXTERNAL_PROMETHEUS_ADDR), port);
		Self { socket_addr, keeper_addr }
	}

	pub fn prometheus_registry(&self) -> Registry {
		let keeper_addr_str = self.keeper_addr.to_owned();
		let param = std::iter::once((String::from("keeper"), keeper_addr_str)).collect();
		Registry::new_custom(None, Some(param)).expect("this can only fail if the prefix is empty")
	}
}

pub(crate) mod utils {
	use super::*;
	pub fn register<T: Clone + Collector + 'static>(
		metric: T,
		registry: &Registry,
	) -> std::result::Result<T, PrometheusError> {
		registry.register(Box::new(metric.clone()))?;
		Ok(metric)
	}

	async fn request_metrics(
		req: Request<Body>,
		registry: Registry,
	) -> std::result::Result<Response<Body>, Error> {
		if req.uri().path() == "/metrics" {
			let metric_families = registry.gather();
			let mut buffer = vec![];
			let encoder = TextEncoder::new();
			encoder.encode(&metric_families, &mut buffer).unwrap();

			Response::builder()
				.status(StatusCode::OK)
				.header("Content-Type", encoder.format_type())
				.body(Body::from(buffer))
				.map_err(Error::Http)
		} else {
			Response::builder()
				.status(StatusCode::NOT_FOUND)
				.body(Body::from("Not found."))
				.map_err(Error::Http)
		}
	}

	/// Initializes the metrics context, and starts an HTTP server
	/// to serve metrics.
	pub async fn init_prometheus(port: u16, registry: Registry) -> std::result::Result<(), Error> {
		let prometheus_addr = SocketAddr::new(IpAddr::from(EXTERNAL_PROMETHEUS_ADDR), port);
		let listener = tokio::net::TcpListener::bind(&prometheus_addr)
			.await
			.map_err(|_| Error::PortInUse(prometheus_addr))?;

		init_prometheus_with_listener(listener, registry).await
	}

	/// Init prometheus using the given listener.
	async fn init_prometheus_with_listener(
		listener: tokio::net::TcpListener,
		registry: Registry,
	) -> std::result::Result<(), Error> {
		let listener = hyper::server::conn::AddrIncoming::from_listener(listener)?;
		log::info!("〽️ Prometheus exporter started at {}", listener.local_addr());

		let service = make_service_fn(move |_| {
			let registry = registry.clone();

			async move {
				Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
					request_metrics(req, registry.clone())
				}))
			}
		});

		let server = Server::builder(listener).serve(service);

		let result = server.await.map_err(Into::into);

		result
	}
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// Hyper internal error.
	#[error(transparent)]
	Hyper(#[from] hyper::Error),

	/// Http request error.
	#[error(transparent)]
	Http(#[from] hyper::http::Error),

	/// i/o error.
	#[error(transparent)]
	Io(#[from] std::io::Error),

	#[error("Prometheus port {0} already in use.")]
	PortInUse(SocketAddr),
}
