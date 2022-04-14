use env_logger::Env;
use structopt::StructOpt;
use command::Opt;
use keeper_primitives::Error;

mod command;
mod entry;
mod tasks;



#[tokio::main]
async fn main() -> std::result::Result<(), Error> {
	// use default log level if it was not set
	env_logger::init_from_env(Env::default().default_filter_or("info"));
	log::info!("running...");

	let opt = Opt::from_args();
	match opt {
		Opt::Start { options } => entry::start(options).await?,
	}
	Ok(())
}
