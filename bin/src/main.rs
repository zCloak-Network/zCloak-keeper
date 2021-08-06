use structopt::StructOpt;

use crate::utils::command::Opt;


mod utils;
mod handler;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    match opt {
        Opt::Server { option } => {
            handler::handle_server(option).await?;
        }
        Opt::Task { server, command } => {
            handler::handle_task(server, command).await?;
        }
    }
    Ok(())
}

