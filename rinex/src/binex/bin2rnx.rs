//! BINEX to RINEX deserialization
use std::io::Read;

use crate::prelude::Rinex;

/// BIN2RNX can deserialize a BINEX stream to RINEX Tokens.
pub struct BIN2RNX<R: Read> {
    /// BINEX decoder
    decoder: Decoder<R>,
}

impl Iterator for BIN2RNX {
    type Item = Option<String>;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl BIN2RNX {
    /// Creates a new [BIN2RNX] working from [Read]able interface.
    /// It will stream Tokens as long as the interface is alive.
    pub fn new<R: Read>(r: R) -> Self {
        Self {
            decoder: Decoder::new(r),
        }
    }
}
