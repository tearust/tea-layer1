#!/bin/sh

cargo +nightly-2020-10-06 clippy -- -A clippy::unnecessary_mut_passed -Z unstable-options