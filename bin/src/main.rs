use structopt::StructOpt;
mod command;
use crate::command::Opt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //TODO: init config
    let opt = Opt::from_args();
    match opt {
        Opt::Server { options } => {
            // TODO:
        },
    }
    Ok(())
}

