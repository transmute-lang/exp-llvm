#!/usr/bin/env bash

pushd llvm || exit 1

wget https://github.com/llvm/llvm-project/archive/refs/tags/llvmorg-18.1.8.tar.gz
tar -xzvf llvmorg-18.1.8.tar.gz

pushd llvm-project-llvmorg-18.1.8 || exit 1
cmake -S llvm -B build -G Ninja -DLLVM_INSTALL_UTILS=ON -DCMAKE_INSTALL_PREFIX=$(pwd)/../llvm-18.1 -DCMAKE_BUILD_TYPE=Release
cmake --build build
cmake --install build

popd || exit 1
popd || exit 1