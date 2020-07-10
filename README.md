# TeaRust Blockchain

## Build

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./scripts/init.sh
```

Build Wasm and native code:

```bash
cargo build --release
```

## Run

### Single Node Development Chain

Purge any existing developer chain state:

```bash
./target/release/tea purge-chain --dev
```

Start a development chain with:

```bash
./target/release/tea --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

### Multi-Node Local Testnet

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

## Initial tea node
> use ed25519 key pair

1. tea_id: df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b
private key: 5579a3c220146f0caaab49b884de505098b89326970b929d781cf4a65445a917df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b

2. tea_id: c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696596
private key: 119c37b9aa65572ad9e24dd49c4f4da5330fe476f3313c560ffc67888f92b758c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696596

3. tea_id: 2754d7e9c73ced5b302e12464594110850980027f8f83c469e8145eef59220b6
private key: 2b0af8d507eee7175290dad578f734a4936b091a05bda5abe5052a104a5936502754d7e9c73ced5b302e12464594110850980027f8f83c469e8145eef59220b6

4. tea_id: c9380fde1ba795fc656ab08ab4ef4482cf554790fd3abcd4642418ae8fb5fd52
private key: 882a976b9990228c28e407c42c0da4a9beff8979f080c0a5edefc243a0d51b01c9380fde1ba795fc656ab08ab4ef4482cf554790fd3abcd4642418ae8fb5fd52
