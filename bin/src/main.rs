use structopt::StructOpt;

mod command;
mod entry;
mod tasks;

use command::Opt;
use keeper_primitives::Error;

#[tokio::main]
async fn main() -> std::result::Result<(), Error> {
	env_logger::init();
	log::info!("running...");

	let opt = Opt::from_args();
	match opt {
		Opt::Start { options } => entry::start(options).await?,
	}
	Ok(())
}
