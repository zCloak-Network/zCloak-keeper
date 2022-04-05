use structopt::StructOpt;

mod command;
mod entry;
mod entry_mock;

use command::Opt;
use keeper_primitives::Error;

#[tokio::main]
async fn main() -> std::result::Result<(), Error> {
	env_logger::init();
	log::info!("running...");
	//TODO: init config
	let opt = Opt::from_args();
	match opt {
		Opt::Start { options } => entry_mock::start(options).await?,
	}
	Ok(())
}
