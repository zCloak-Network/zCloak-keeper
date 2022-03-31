use structopt::StructOpt;

mod command;
mod entry;

use command::Opt;
use keeper_primitives::Error;

#[tokio::main]
async fn main() -> std::result::Result<(), Error>{
    //TODO: init config
    let opt = Opt::from_args();
    match opt {
        Opt::Start { options } => {
            entry::start(options).await?
        },
    }
    Ok(())
}

