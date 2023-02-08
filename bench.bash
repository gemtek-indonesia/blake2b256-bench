#!/usr/bin/env bash

set -e
export RUSTFLAGS="-C target-cpu=native -C codegen-units=1 -C opt-level=3 -C link-args=-s -C target-feature=+neon,+aes,+sha2,+fp16" 

echo -e "\n\n\033[1;36m[Without Mimalloc]\033[0m"
cargo build --release -q >/dev/null 2>&1
head -c 16M /dev/urandom > /tmp/bigfile-16MB
target/release/blake2b256-bench --filepath /tmp/bigfile-16MB

echo -e "\n\n\033[1;36m[With Mimalloc]\033[0m"
cargo build --release -q --features="tlsmalloc" >/dev/null 2>&1
head -c 16M /dev/urandom > /tmp/bigfile-16MB
target/release/blake2b256-bench --filepath /tmp/bigfile-16MB

unset RUSTFLAGS
