## Build for dev local with Docker

```
. docker/build-image.sh
. docker/run.sh

```

## Local build and test on Mac

Install toolchain
- rustup toolchain install nightly-2020-10-06
- rustup target add wasm32-unknown-unknown --toolchain nightly-2020-10-06

Build
```
. build-local.sh
```

Run
```
./target/debug/tea-layer1 --dev --tmp
```
