use crate::utils::command::Opt;
use structopt::StructOpt;

mod handler;
mod route;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	utils::utils::init()?;
	let opt = Opt::from_args();
	match opt {
		Opt::Server { options } => {
			handler::handle_server(options).await?;
		},
		Opt::Task { server, command } => {
			handler::handle_task(server, command).await?;
		},
		Opt::Kv { server, namespace, command } => {
			handler::handle_kv(server, namespace, command).await?;
		},
		Opt::Crypto(command) => {
			handler::handle_crypto(command).await?;
		},
		Opt::Tools { command } => {
			handler::handle_tools(command).await?;
		},
	}
	Ok(())
}
