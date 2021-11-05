### Config
1. `cp .config/task-moonbeam.toml ~/.zcloak-server`
    '~/.zcloak-server' can replace with your local path
2. `vim task-moonbeam.toml`
```
[moonbeam]
url = "ws://127.0.0.1:9944"

[ipfs]
url_index = "https://ipfs.infura.io:5001/api/v0/cat?arg="

[contract]
address = "0x3cB0048299bcA8438B244A50cCA30eC7d7C3564A"
topics = ["6d52a2695eb8abea86b820937079d5dddfdfaefc969a3a132f1d7315e020acf7"]

```
moonbeam.url is moonbeam parachain network in mode -dev , localhost:ws-port
ipfs.url_index is verify proof url saved on ipfs network.
contract.address is the smart contract address deployed on moonbeam network.
contract.topics is the array contain the event log3 value. the event is needed to subscribe.

1. `cargo build --release`

### Run
```
./target/release/zcloak-server server --base-path ~/.zcloak-server
```

### Verifier
- you can get smart contract code at this project [z-profile](https://github.com/zCloak-Network/z-profile.git)
- you can use moonbeam tutorial about [remix](https://docs.moonbeam.network/cn/builders/interact/remix/)
- start the zcloak worker server,then use remix to summit transaction

### Notice
The file name 'ZeroKnowlegeProof.json' is smart contract abi file. It must be consistent with smart contract.