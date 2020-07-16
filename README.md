<<<<<<< HEAD
[![Try on playground](https://img.shields.io/badge/Playground-node_template-brightgreen?logo=Parity%20Substrate)](https://playground-staging.substrate.dev/?deploy=node-template)

# Substrate Node Template

A new FRAME-based Substrate node, ready for hacking :rocket:

## Local Development

Follow these steps to prepare your local environment for Substrate development :hammer_and_wrench:

### Simple Method

You can install all the required dependencies with a single command (be patient, this can take up
to 30 minutes).

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast
```

### Manual Method
=======
# TeaRust Blockchain
>>>>>>> rpc

Manual steps for Linux-based systems can be found below; you can
[find more information at substrate.dev](https://substrate.dev/docs/en/knowledgebase/getting-started/#manual-installation).

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./scripts/init.sh
```

### Build

Once you have prepared your local development environment, you can build the node template. Use this
command to build the [Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution)
and [native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

```bash
cargo build --release
```

## Playground [![Try on playground](https://img.shields.io/badge/Playground-node_template-brightgreen?logo=Parity%20Substrate)](https://playground-staging.substrate.dev/?deploy=node-template)

[The Substrate Playground](https://playground-staging.substrate.dev/?deploy=node-template) is an
online development environment that allows you to take advantage of a pre-configured container
with pre-compiled build artifacts :woman_cartwheeling:

## Run

### Single Node Development Chain

Purge any existing developer chain state:

```bash
./target/release/node-template purge-chain --dev
```

Start a development chain with:

```bash
./target/release/node-template --dev
```

Detailed logs may be shown by running the node with the following environment variables set:
`RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action, refer to
[our Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).

### Run in Docker

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.

```bash
./scripts/docker_run.sh
```

This command will firstly compile your code, and then start a local development network. You can
also replace the default command (`cargo build --release && ./target/release/node-template --dev --ws-external`)
by appending your own. A few useful ones are as follow.

```bash
<<<<<<< HEAD
# Run Substrate node without re-compiling
./scripts/docker_run.sh ./target/release/node-template --dev --ws-external

# Purge the local dev chain
./scripts/docker_run.sh ./target/release/node-template purge-chain --dev

# Check whether the code is compilable
./scripts/docker_run.sh cargo check
```
=======
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
>>>>>>> rpc
