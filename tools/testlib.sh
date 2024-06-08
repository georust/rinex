#! /bin/sh
cargo fmt
cargo test -- --nocapture
cargo test --all-features -- --nocapture
./tools/builddoc.sh
