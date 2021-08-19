### Config
1. `cp .config/task-zcloak-substrate.toml ~/.zcloak-server`
    '~/.zcloak-server' can replace with your local path
2. `vim task-zcloak-substrate.toml`
```
[zcloak]
url = "ws://127.0.0.1:9944"
private_key = "0x..."

[ipfs]
url_index = "https://ipfs.infura.io:5001/api/v0/cat?arg="
```
zcloak.url is zCloak Network localhost:ws-port,private_key is account private key which used to commit a verify result to zCloak Network .
ipfs.url_index is verify proof url saved on ipfs network.

3. `cargo build --release`

### Run
```
./target/release/zcloak-server server --base-path ~/.zcloak-server
```

### Verifier
use polkadot.js to contect  zCloak-Networ local testnet with development model.then create verify task through starts verifier seperate pallet.