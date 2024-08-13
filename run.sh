#!/usr/bin/env bash

export LLVM_SYS_180_PREFIX="$(pwd)/llvm/llvm-18.1/"

cargo run
ar rcs target/libsum.a target/sum.o
cc main.c -lsum -Ltarget -o target/main
target/main