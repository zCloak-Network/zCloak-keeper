use std::path::PathBuf;
use structopt::StructOpt;

use verify_traits::server::config::ConfigFormat;

#[derive(Debug, StructOpt)]
#[structopt(name = "verify", about = "zCloak server")]
pub enum Opt {

    Task {
        #[structopt(long, default_value = "http://127.0.0.1:3088")]
        server: String,
        #[structopt(flatten)]
        command: TaskCommand,
    },

    Server {
        #[structopt(flatten)]
        Option: ServerOptions,
    },
}

#[derive(Debug, StructOpt)]
pub enum TaskCommand {

}

#[derive(Debug, clone, StructOpt)]
pub struct ServerOptions {

    ///zCloak server listen host 
    #[structopt(short, long, default_value = "http://127.0.0.1")]
    pub host: String,

    ///zCloak server listen port
    #[structopt(short, long, default_value = "3088")]
    pub port: u32,

    #[structopt(long, parse(from_os_str))]
    pub base_path: Option<PathBuf>,
}