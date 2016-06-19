#! /usr/bin/env sh

set -eu

rust_ios_install() {
    ios_stdlib="rust-std-1.9.0-${1}-apple-ios"
    curl -O "https://static.rust-lang.org/dist/${ios_stdlib}.tar.gz"
    tar xzf "${ios_stdlib}.tar.gz"
    "./${ios_stdlib}/install.sh" --prefix=$(rustc --print sysroot)
}

gnustep_install() {
    git clone https://github.com/gnustep/libobjc2.git
    mkdir libobjc2/build
    cd libobjc2/build
    export CC="clang"
    export CXX="clang++"
    cmake -DCMAKE_INSTALL_PREFIX:PATH=$HOME/libobjc2_staging ../
    make install
}

for arch in $IOS_ARCHS; do
    rust_ios_install "$arch"
done

if [ -n "$IOS_ARCHS" ]; then
    curl -LO https://github.com/SSheldon/rust-test-ios/releases/download/0.1.1/rust-test-ios
    chmod +x rust-test-ios
fi

if [ "$TRAVIS_OS_NAME" = "linux" ]; then
    gnustep_install
fi
