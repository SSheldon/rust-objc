git clone https://github.com/gnustep/libobjc2.git
mkdir libobjc2/build
pushd libobjc2/build
cmake -DCMAKE_INSTALL_PREFIX:PATH=$HOME/libobjc2_staging ../
make install
export CPATH=$HOME/libobjc2_staging/include:$CPATH
export LIBRARY_PATH=$HOME/libobjc2_staging/lib:$LIBRARY_PATH
export LD_LIBRARY_PATH=$HOME/libobjc2_staging/lib:$LD_LIBRARY_PATH
popd
