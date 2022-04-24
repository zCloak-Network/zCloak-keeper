use command::Opt;
use env_logger::Env;
use keeper_primitives::Error;
use structopt::StructOpt;
use futures::FutureExt;

mod command;
mod entry;
mod tasks;
mod runner;

#[tokio::main]
async fn main() -> std::result::Result<(), Error> {
	// use default log level if it was not set
	env_logger::init_from_env(Env::default().default_filter_or("info"));
	log::info!("running...");

	let opt = Opt::from_args();
	match opt {
		Opt::Start { options } => {
			let f = entry::start(options);
			let f = f.fuse();
			runner::main(f).await?;
		},
	}
	Ok(())
}
