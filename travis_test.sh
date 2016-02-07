#! /usr/bin/env sh

set -eu

if [ -z "$IOS_ARCHS" ]; then
    cargo build --verbose --features "$FEATURES"
    cargo test --verbose --features "$FEATURES"
else
    make -C "xtests" RUST_ARCHS="$IOS_ARCHS"
    cd ios-tests
    xcodebuild \
        -project RustObjCTests.xcodeproj \
        -scheme RustObjC \
        -destination 'platform=iOS Simulator,name=iPhone 5' \
        -destination 'platform=iOS Simulator,name=iPhone 5s' \
        test
fi
