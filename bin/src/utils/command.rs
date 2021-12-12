use std::path::PathBuf;
use structopt::StructOpt;

use server_traits::server::config::ConfigFormat;

#[derive(Debug, StructOpt)]
#[structopt(name = "verify", about = "zCloak server")]
pub enum Opt {
	/// Task Manager
	Task {
		/// zCloak Server host
		#[structopt(long, default_value = "http://127.0.0.1:3088")]
		server: String,
		#[structopt(flatten)]
		command: TaskCommand,
	},
	/// The kv db storage operation
	Kv {
		/// zCloak Server host
		#[structopt(long, default_value = "http://127.0.0.1:3088")]
		server: String,
		/// The namespace of storage
		#[structopt(long, short)]
		namespace: Option<String>,
		#[structopt(flatten)]
		command: KvCommand,
	},

	///start zCloak Server
	Server {
		#[structopt(flatten)]
		options: ServerOptions,
	},

	///use zCloak tools
	Tools {
		#[structopt(flatten)]
		command: ToolsCommand,
	},

	/// Crypto help command
	Crypto(CryptoCommand),
}

#[derive(Debug, StructOpt)]
pub enum CryptoCommand {
	/// encrypt a value
	Encrypt {
		#[structopt(flatten)]
		options: CryptoOptions,
	},
	/// decrypt a value
	Decrypt {
		#[structopt(flatten)]
		options: CryptoOptions,
	},
}

#[derive(Debug, StructOpt)]
pub enum KvCommand {
	/// Put Key-Value to server database
	Put {
		/// keys and values one by one
		#[structopt()]
		kvs: Vec<String>,
	},
	/// Get Key-Value from server
	Get {
		/// Get a value by key
		#[structopt()]
		keys: Vec<String>,
	},
	/// List server database
	List {
		/// List by sorted
		#[structopt(short, long)]
		sorted: bool,
	},
	/// Remove a Key-Value from server
	Remove {
		/// Remove a value by key
		#[structopt()]
		keys: Vec<String>,
	},
}

#[derive(Debug, StructOpt)]
pub enum TaskCommand {
	List,
	/// Start a task
	Start {
		/// Options of task control
		#[structopt(flatten)]
		options: TaskControlOptions,
	},
	/// Restart a task
	Restart {
		/// Options of task control
		#[structopt(flatten)]
		options: TaskControlOptions,
	},
	/// Stop a running task
	Stop {
		/// The task name
		#[structopt(short, long)]
		name: String,
	},
	/// Execute task command
	Exec {
		/// Options of task execute
		#[structopt(flatten)]
		options: TaskExecuteOptions,
	},
	/// Show config template
	ConfigTemplate {
		/// The task name
		#[structopt(short, long)]
		name: String,
		/// The config format, supports [toml|json|yml]
		#[structopt(long, default_value = "toml")]
		format: ConfigFormat,
	},
	/// Set password for this task to decrypt task config.
	SetPassword {
		/// The task name
		#[structopt(short, long)]
		name: String,
		/// Is store password to database. if store it, the next time will load this.
		#[structopt(short, long)]
		store: bool,
	},
}

#[derive(Debug, StructOpt)]
pub enum ToolsCommand {
	///tools keccak256 computs the hash of input
	Keccak256 {
		///inputs
		#[structopt(short, long)]
		inputs: String,
	},
	Decrypt {
		///path
		#[structopt(long)]
		path: String,

		///password
		#[structopt(long)]
		password: String,
	},
	NewAccount {
		///password
		#[structopt(short, long)]
		password: String,
	},
}

#[derive(Clone, Debug, StructOpt)]
pub struct TaskExecuteOptions {
	/// The task name
	#[structopt(short, long)]
	pub name: String,
	/// The api of task
	#[structopt(short, long)]
	pub api: String,
	/// The parameters of this api
	#[structopt(short, long, default_value = "")]
	pub param: Vec<String>,
	/// The password to decrypt config if necessary
	#[structopt(short = "P", long)]
	pub password: bool,
}

#[derive(Clone, Debug, StructOpt)]
pub struct TaskControlOptions {
	/// The task name
	#[structopt(short, long)]
	pub name: String,
	/// The config format, supports [toml|json|yml]
	#[structopt(long, default_value = "toml")]
	pub format: ConfigFormat,
	/// The config file path, When first run this is required, but the server already have this
	/// task config, can be skip this parameter
	#[structopt(short, long)]
	pub config: Option<PathBuf>,
	/// The password to decrypt config if necessary
	#[structopt(short, long)]
	pub password: bool,
	/// Store password to database.
	#[structopt(long)]
	pub store_password: bool,
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
}

#[derive(Clone, Debug, StructOpt)]
pub struct CryptoOptions {
	/// The value your want encrypt or decrypt
	#[structopt(short, long)]
	pub value: String,
}
