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

BINEX is embedded friendty: no-std is supported by default. You can unlock "std" support by compiling
with the `std` option. Read last paragraph for related consequences.

## Message Decoding

Use the BINEX `Decoder` to decode messages from a `Readable` interface:

```rust
use std::fs::File;
use binex::prelude::{Decoder, Error};

// Create the Decoder:
//  * works from our local source
//  * needs to be mutable due to iterating process
let mut fd = File::open("../test_resources/BIN/mfle20190130.bnx")
    .unwrap();

let mut decoder = Decoder::new(fd);

// Consume data stream
loop {
    match decoder.next() {
        Some(Ok(msg)) => {
            
        },
        Some(Err(e)) => match e {
            Error::IoError => {
                // any I/O error should be handled
                // and user should react accordingly,
            },
            Error::ReversedStream | Error::LittleEndianStream => {
                // this library is currently limited:
                //  - reversed streams are not supported yet
                //  - little endian streams are not supported yet
            },
            Error::InvalidStartofStream => {
                // other errors give meaningful information
            },
            _ => {},
        },
        None => {
            // reacehed of stream!
            break;
        },
    }
}
```

## Message forging

The BINEX library allows easy message forging. Each message can be easily encoded and then
streamed into a `Writable` interface:

```rust
```

## BINEX and physical applications / high level operations

BINEX is not processing nor calculation oriented. It supports `no-std` which is very limiting
and allows embedding BINEX on STM32 or similarly limited platforms. The scope of this library
is to encode and decode messages: we don't not interprate the GNSS streams.

If you want access to higher level operations that permit physical applications, you need to move on
to the RINEX library (which does not support `no-std`) and compile it with the `binex` compilation option.

* it will unlock the RINEX to BINEX encoding function (:warning: Work In Progress :warning:)
* it will unlock the BINEX to RINEX: high level interpretation (:warning: Work In Progress :warning:)
* you can then access all the RINEX methods, mostly
   - Interprate and manipulate Ephemeris frames, especially using the Nyx/ANISE toolkit
   - Encode to binary Ephemeris that were processed using Nyx/ANISE 
   - Interprate signal observations and process them (mainly, signal combination)

## Licensing

Licensed under either of:

* Apache Version 2.0 ([LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0))
* MIT ([LICENSE-MIT](http://opensource.org/licenses/MIT)
