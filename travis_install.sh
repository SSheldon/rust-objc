#! /usr/bin/env sh

set -eu

rust_ios_install() {
    ios_stdlib="rust-std-nightly-${1}-apple-ios"
    curl -O "http://static.rust-lang.org/dist/${ios_stdlib}.tar.gz"
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

if [ "$TRAVIS_OS_NAME" = "linux" ]; then
    gnustep_install
fi
