# BINEX

[![Rust](https://github.com/georust/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/georust/rinex/actions/workflows/rust.yml)
[![Rust](https://github.com/georust/rinex/actions/workflows/daily.yml/badge.svg)](https://github.com/georust/rinex/actions/workflows/daily.yml)

[![crates.io](https://img.shields.io/crates/v/binex.svg)](https://crates.io/crates/binex)
[![crates.io](https://docs.rs/binex/badge.svg)](https://docs.rs/binex/badge.svg)

BINEX is a simple library to decode and encode BINEX messages.  
BINEX stands for BINary EXchange and is the "real time" stream oriented
version of the RINEX format.

RINEX is a readable text format which is based on line termination and allows describing
from the minimum requirement for GNSS navigation up to very precise navigation and
other side GNSS applications.

BINEX is a binary stream (non readable) conversion to that, dedicated to GNSS receivers and hardware interfacing.  
Like RINEX, it is an open source format, the specifications are described by
[UNAVCO here](https://www.unavco.org/data/gps-gnss/data-formats/binex).

This library allows easy message encoding and decoding, and aims at providing seamless
convertion from RINEX back and forth.

## Message Decoding

Use the BINEX `Decoder` to decode messages from a `Readable` interface:

```rust
```

## Message forging

The BINEX library allows easy message forging. Each message can be easily encoded and then
streamed into a `Writable` interface:

```rust
```

## Licensing

Licensed under either of:

* Apache Version 2.0 ([LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0))
* MIT ([LICENSE-MIT](http://opensource.org/licenses/MIT)
