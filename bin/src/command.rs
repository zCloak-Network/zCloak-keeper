use structopt::StructOpt;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(name = "zcloak Keeper", about = "zCloak keeper node start config")]
pub enum Opt {

    ///start zCloak Server
    Start {
        #[structopt(flatten)]
        options: StartOptions,
    },
}

#[derive(Debug, Clone, StructOpt)]
pub struct StartOptions {

    ///The zCloak server config or data base path
    #[structopt(long, parse(from_os_str))]
    pub config: Option<PathBuf>,

    /// The starting block number of scanning node events.
    #[structopt(short, long)]
    pub start_number: Option<u64>,
}