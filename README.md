# zCloak Keeper

[![GitHub issues](https://img.shields.io/github/issues/zcloak-network/zcloak-keeper)](https://github.com/zCloak-network/zCloak-keeper/issues) [![GitHub forks](https://img.shields.io/github/forks/zcloak-network/zcloak-keeper)](https://github.com/zCloak-Network/zCloak-keeper/network) [![GitHub license](https://img.shields.io/github/license/zcloak-network/zcloak-keeper)](https://github.com/zCloak-Network/zCloak-keeper/blob/main/LICENSE)

zCloak Keeper is the verifier client which provides Zero-knowledge Proof for many chains,such as zCloak Network, Polkadot Network etc(base on substrate frame).

zCloak Keeper will integrate with multiple chains in the future.

## Components
**component-moonbeam**
- scan moonbeam addProof events
- submit transaction back to moonbeam

**component-ipfs**
- query raw proof bytes on ipfs and decode it to `StarkProof`
- stark verify the proof and output the verify result

**component-kilt**
- check the validity of the credential through rootHash

## Process
The workflow of zCloak keeper is:
1. keep scanning AddProof event on moonbeam
2. get the cid out of the event scanned and fetch the raw proof bytes
3. parse the raw proof bytes into `StarkProof`
4. verify the `StarkProof` with StarkVM verifier and output `rootHash` and `isPassed`
5. query the attester address and the validity of user's credential from Kilt Network
6. submit the validity, attester and verify result back to moonbeam

## Todos in near future
-[ ] introduce database
-[ ] enhance message queue utility

## Future Plan
- integrate with other evm-compatible chains
- introduce p2p and raw consensus
- introduce threshold signature


## Installation

### Download from GitHub
Download the binary from [main branch](https://github.com/zCloak-Network/zCloak-keeper).

### Build from source
```
git clone git@github.com:zCloak-Network/zCloak-keeper.git
cd zCloak-keeper/
cargo build --release
```

## Surpport Chain
| chain name | frame | doc   |
|------------| ----- |-------|
| Moonbeam   | substrate | [WIP] |

Some Networks which based on substrate want to provide Zero-knowledge Proof should dependend starks verifier seperate pallet in runtime.


## Usage
zcloak-keeper --help

```
zcloak Keeper 0.1.0
zCloak keeper node start config

USAGE:
    zcloak-keeper <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help     Prints this message or the help of the given subcommand(s)
    start    start zCloak Server
```

start zCloak Keeper

for instance:
```bash
zcloak-keeper start --config ./config.json --cache-dir ./data --start-number 100
```

```bash
$ zcloak-keeper start --help
```

```bash
zcloak-keeper-start 0.1.0
start zCloak Server

USAGE:
    zcloak-keeper start [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
OPTIONS:
        --cache-dir <cache-dir>          The zCloak keeper node msg queue cache directory
        --config <config>                The zCloak keeper node config file path
    -s, --start-number <start-number>    The starting block number of scanning node events
```

- `--config` the path of zCloak keeper's config file
- `--cache-dir` the directory path which zCloak keeper cache the message queue files
- `-s` or `--start-number` where to start the moonbeam series networks scan