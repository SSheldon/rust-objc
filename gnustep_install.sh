#! /usr/bin/env sh

set -ev

git clone https://github.com/gnustep/libobjc2.git
mkdir libobjc2/build
cd libobjc2/build
export CC="clang"
export CXX="clang++"
cmake -DCMAKE_INSTALL_PREFIX:PATH=$HOME/libobjc2_staging ../
make install
