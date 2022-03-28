use structopt::StructOpt;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(name = "verify", about = "zCloak server")]
pub enum Opt {

    ///start zCloak Server
    Server {
        #[structopt(flatten)]
        options: ServerOptions,
    },
}

#[derive(Debug, Clone, StructOpt)]
pub struct ServerOptions {
    ///zCloak server listen host
    #[structopt(short, long, default_value = "127.0.0.1")]
    pub host: String,

    ///zCloak server listen port
    #[structopt(short, long, default_value = "3088")]
    pub port: u32,

    ///The zCloak server config or data base path
    #[structopt(long, parse(from_os_str))]
    pub base_path: Option<PathBuf>,

    /// The starting block number of scanning node events.
    #[structopt(short, long)]
    pub start_number: Option<u64>,
}