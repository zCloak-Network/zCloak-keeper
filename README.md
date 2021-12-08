# zCloak Worker

[![GitHub issues](https://img.shields.io/github/issues/zcloak-network/zcloak-worker)](https://github.com/zCloak-network/zCloak-worker/issues) [![GitHub forks](https://img.shields.io/github/forks/zcloak-network/zcloak-worker)](https://github.com/zCloak-Network/zCloak-worker/network) [![GitHub license](https://img.shields.io/github/license/zcloak-network/zcloak-worker)](https://github.com/zCloak-Network/zCloak-worker/blob/main/LICENSE)

zCloak Worker is Verify Server Client which provides Zero-knowledge Proof for many chains,such as zCloak Network,Polkadot Network etc(base on substrate frame).

zCloak Worker will multiple chains in the future.

## Installation

### Download from GitHub
Download the binary from [main branch](https://github.com/zCloak-Network/zCloak-worker).

### Build from source
```
git clone git@github.com:zCloak-Network/zCloak-worker.git
cd zCloak-worker/
cargo build --release
```

## Surpport Chain
| chain name | frame | doc |
| ---------- | ----- | ----- |
| zCloak Network | substrate | [Usage](./tast/../task/task-zcloak-substrate/docs/Usage.md) |

Some Networks which based on substrate want to provide Zero-knowledge Proof should dependend starks verifier seperate pallet in runtime.


## Usage
zcloak-worker --help

```
$ zcloak-worker
verify 0.1.0
zCloak worker

USAGE:
    zcloak-worker <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    crypto    Crypto help command
    help      Prints this message or the help of the given subcommand(s)
    kv        The kv db storage operation
    server    start zCloak Worker Server
    task      Task Manager
```

start zCloak Worker

```
$ zcloak-worker server
```

```
$zcloak-worker server --help
zcloak-worker-server 0.1.0
start zCloak Worker

USAGE:
    zcloak-worker server [OPTIONS]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --base-path <base-path>    The zCloak worker config or data base path
    -h, --host <host>              zCloak worker listen host [default: 127.0.0.1]
    -p, --port <port>              zCloak worker listen port [default: 3088]
```

- `--base-path` zCloak Worker's config 、database will store in this  path.
- `--host` `--port` the zCloak Worker host and port