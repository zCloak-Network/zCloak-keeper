# zCloak Server

[![GitHub issues](https://img.shields.io/github/issues/zCloak-network/zCloak-server)](https://github.com/zCloak-network/zCloak-server/issues) [![GitHub forks](https://img.shields.io/github/forks/zCloak-Network/zCloak-server)](https://github.com/zCloak-Network/zCloak-server/network) [![GitHub license](https://img.shields.io/github/license/zCloak-Network/zCloak-server)](https://github.com/zCloak-Network/zCloak-server/blob/main/LICENSE)

zCloak Server is Verify Server Client which provides Zero-knowledge Proof for many chains,such as zCloak Network,Polkadot Network etc(base on substrate frame).

zCloak Server will support other chains in the future.

## Installation

### Download from GitHub
Download the binary from [main branch](https://github.com/zCloak-Network/zCloak-server).

### Build from source
```
git clone git@github.com:zCloak-Network/zCloak-server.git
cd zCloak-server/
cargo build --release
```

## Surpport Chain
| chain name | frame | doc |
| ---------- | ----- | ----- |
| zCloak Network | substrate | [Usage](./tast/../task/task-zcloak-substrate/docs/Usage.md) |

Some Networks which based on substrate want to provide Zero-knowledge Proof should dependend starks verifier seperate pallet in runtime.


## Usage
zcloak-server --help

```
$ zcloak-server
verify 0.1.0
zCloak server

USAGE:
    zcloak-server <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    crypto    Crypto help command
    help      Prints this message or the help of the given subcommand(s)
    kv        The kv db storage operation
    server    start zCloak Server
    task      Task Manager
```

start zCloak Server

```
$ zcloak-server server
```

```
$zcloak-server server --help
zcloak-server-server 0.1.0
start zCloak Server

USAGE:
    zcloak-server server [OPTIONS]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --base-path <base-path>    The zCloak server config or data base path
    -h, --host <host>              zCloak server listen host [default: 127.0.0.1]
    -p, --port <port>              zCloak server listen port [default: 3088]
```

- `--base-path` zCloak Server's config 、database will store in this  path.
- `--host` `--port` the zCloak Server host and port