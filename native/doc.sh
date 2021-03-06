#!/bin/bash

set -e

mkdir -p target/doc
cp -a ~/.rustup/toolchains/`rustup show active-toolchain`/share/doc/rust/html/* target/doc
cargo doc --open
cargo watch -x doc
