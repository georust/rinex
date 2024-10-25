//! RINEX to BINEX serialization
use std::io::Read;

use crate::prelude::Rinex;

use binex::prelude::{Encoder, Message};

/// RNX2BIN can serialize a RINEX to a stream of BINEX [Message]s
pub struct RNX2BIN<W: Write> {
    /// BINEX encoder
    encoder: Encoder<W>,
}

impl Iterator for RNX2BIN {
    type Item = Option<Message>;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl RNX2BIN {
    /// Creates a new [RNX2BIN].
    pub fn new<W: Write>(w: W) -> Self {
        Self {
            encoder: Encoder::new(r),
        }
    }
}
