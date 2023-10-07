#!/bin/sh
cargo clippy \
    --fix \
    --allow-dirty \
    -- -Dclippy::perf
