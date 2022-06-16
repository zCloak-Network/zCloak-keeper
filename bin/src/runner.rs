use futures::{future, future::FutureExt, pin_mut, select, Future};
use crate::error::Error;

#[cfg(target_family = "unix")]
pub async fn run_until_exit<F>(func: F) -> std::result::Result<(), Error>
where
	F: Future<Output = std::result::Result<(), Error>> + future::FusedFuture,
{
	use tokio::signal::unix::{signal, SignalKind};

	let mut stream_int = signal(SignalKind::interrupt())?;
	let mut stream_term = signal(SignalKind::terminate())?;

	let t1 = stream_int.recv().fuse();
	let t2 = stream_term.recv().fuse();
	let t3 = func;

	pin_mut!(t1, t2, t3);

	select! {
		_ = t1 => {},
		_ = t2 => {},
		res = t3 => res?,
	}

	log::info!("zCloak-Keeper Exit Normally.");

	Ok(())
}

#[cfg(not(unix))]
async fn main<F>(func: F) -> std::result::Result<()>
where
	F: Future<Output = std::result::Result<(), E>> + future::FusedFuture,
{
	use tokio::signal::ctrl_c;

	let t1 = ctrl_c().fuse();
	let t2 = func;

	pin_mut!(t1, t2);

	select! {
		_ = t1 => {},
		res = t2 => res?,
	}

	Ok(())
}
