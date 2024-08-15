#!/usr/bin/env bash

export LLVM_SYS_180_PREFIX="$(pwd)/llvm/llvm-18.1/"

pushd rustlib || exit 1
cargo build
popd || exit 1

cargo run
ar rcs target/libfibo.a target/fibo.o
clang main.c \
  -lm -ldl -lpthread \
  -lfibo -Ltarget  \
  -lexp_llvm_rustlib -Ltarget/debug \
  -o target/main

echo
target/main