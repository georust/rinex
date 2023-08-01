#! /bin/sh
VERBOSE=$1
cargo test --features tests
cargo test --all-features
