#! /bin/sh
RUSTDOCFLAGS="--cfg docrs" \
    cargo +nightly doc --all-features
