//! BIN2RNX and RNX2BIN
use binex::prelude::{Decoder, Message};

use crate::prelude::Rinex;
use std::io::Read;

#[cfg_attr(docsrs, doc(cfg(feature = "binex")))]
impl Rinex {
    /// Serialize [Rinex] as serie of BINEX [Message]s that you can then easily stream.
    pub fn to_binex_stream(&self) -> Box<dyn Iterator<Item = Message>> {
        Box::new([].into_iter())
    }
    /// Deserialize a serie of BINEX [Message]s to a [Rinex]
    pub fn from_binex_stream<R: Read>(&self, r: R) -> Self {
        let _ = Decoder::new(r);
        Self::default()
    }
}
