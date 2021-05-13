#! /bin/sh
#
# cov.sh
# Copyright (C) 2021 Emiliano Firmino <emiliano.firmino@gmail.com>
#
# Distributed under terms of the MIT license.
#


cargo clean

export RUSTFLAGS="-Zinstrument-coverage"
cargo build

export LLVM_PROFILE_FILE="citron.profraw"
cargo test

grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/

open target/debug/coverage/index.html
