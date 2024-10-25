//! RINEX to BINEX serialization
use std::io::Write;

use crate::prelude::Rinex;

use binex::prelude::{Encoder, Message};

/// RNX2BIN can serialize a RINEX to a stream of BINEX [Message]s
pub struct RNX2BIN<W: Write> {
    /// BINEX encoder
    encoder: Encoder<W>,
}

impl<W: Write> Iterator for RNX2BIN<W> {
    type Item = Option<Message>;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<W: Write> RNX2BIN<W> {
    /// Creates a new [RNX2BIN].
    pub fn new(w: W) -> Self {
        Self {
            encoder: Encoder::new(w),
        }
    }
}
