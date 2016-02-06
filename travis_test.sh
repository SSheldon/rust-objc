#! /usr/bin/env sh

set -eu

if [ -z "$IOS_ARCHS" ]; then
    cargo build --verbose --features "$FEATURES"
    cargo test --verbose --features "$FEATURES"
else
    make -C "xtests" RUST_ARCHS="$IOS_ARCHS"
fi
