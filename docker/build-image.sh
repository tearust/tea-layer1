#!/usr/bin/env bash

set -e

echo "*** Start build tearust/tea-layer1:latest ***"

cd $(dirname ${BASH_SOURCE[0]})/..

sh ./docker/build.sh

mkdir -p tmp

cp ./docker/target/release/tea-layer1 tmp

docker build -t tearust/tea-layer1:gluon-0.1.5 .

rm -rf tmp
