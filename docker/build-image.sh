#!/usr/bin/env bash

set -e

echo "*** Start build tearust/tea-layer1:latest ***"

cd $(dirname ${BASH_SOURCE[0]})/..

mkdir tmp

cp ./docker/target/release/tea-layer1 tmp

docker build -t tearust/tea-layer1:latest .

mkdir -rf tmp