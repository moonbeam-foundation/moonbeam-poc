# Archived

This repository has been archived and is read-only. Moonbeam is currently being maintained [here](https://github.com/PureStake/moonbeam).

Moonbeam is an Ethereum-compatible smart contract platform for building interoperable applications on Polkadot.

More information: https://moonbeam.network/

Built by [PureStake](https://purestake.com).

# Moonbeam

A proof of concept for the Moonbeam node implementation. 

Moonbeam is a developer platform for creating DeFi applications that target users and assets on Polkadot, Substrate, and other chains.

Here is a list of things which have been done:
* Cloned substrate-node-template repo as a starting point
* Added contracts module to the runtime at the same revision as the node-template (d1cd01c74e8d5550396cb654f9a3f1b641efdf4c)
* Changed references from node-template to moonbeam throughout.
* Rebase code against updated node-template (40a16efefc070faf5a25442bc3ae1d0ea2478eee) and re-integrat contracts module to get contracts working with latest polkadot-js apps.
* Configured token symbol to be GLMR
* Initial checkin for Moonbeam runtime implementing initial set of functions for Moonbeam Dex proof of concept.

## Download
```bash
git clone https://github.com/PureStake/moonbeam-poc
cd moonbeam-poc
```

## Build
Install Package Dependencies:
```bash
sudo apt install -y cmake pkg-config libssl-dev build-essential git clang libclang-dev
```
Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./scripts/init.sh
```

Build Wasm and native code (this takes a while):

```bash
cargo build --release
```

## Run

### Single node development chain

Purge any existing development chain state:

```bash
./target/release/moonbeam purge-chain --dev
```

Start a development chain with:

```bash
./target/release/moonbeam --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

### Interacting with the node

You can interact with the moonbeam node using the polkadot.js apps interface from chrome.  Navigate to:

[https://polkadot.js.org/apps/#/settings](https://polkadot.js.org/apps/#/settings)

In the first remote node dropdown select "Local Node (Own, 127.0.0.1:9944)" and then hit the save button on the bottom right.

You should now be connected to your locally running node.

### Multi-node local testnet

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units.

Optionally, give each node a name and expose them so they are listed on the Polkadot [telemetry site](https://telemetry.polkadot.io/#/Local%20Testnet).

You'll need two terminal windows open.

We'll start Alice's substrate node first on default TCP port 30333 with her chain database stored locally at `/tmp/alice`. The bootnode ID of her node is `QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR`, which is generated from the `--node-key` value that we specify below:

```bash
cargo run -- \
  --base-path /tmp/alice \
  --chain=local \
  --alice \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --validator
```

In the second terminal, we'll start Bob's substrate node on a different TCP port of 30334, and with his chain database stored locally at `/tmp/bob`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

```bash
cargo run -- \
  --base-path /tmp/bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR \
  --chain=local \
  --bob \
  --port 30334 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --validator
```

Additional CLI usage options are available and may be shown by running `cargo run -- --help`.

