Ublox-rnx 
=========

Efficient RINEX data production by connecting to a `Ublox` receiver

```shell
ublox-rnx --port /dev/ttyUSB0 --baud 9600 --obs --nav
```

This tool currently only works on GPS constellation.
Other constellations will be supported in future releases.

## Requirements:

* `libudev-dev`

## Cross compilation

For instance on ARM7 using the Cargo ARM7 configuration 
(I recommend using `rustup` to install the configuration):

```shell
rustup target add armv7-unknown-linux-gnueabihf
cargo build --release \ # release mode: reduce binary size
    --target armv7-unknown-linux-gnueabihf
```
