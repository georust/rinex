//! BINEX to RINEX deserialization
use std::io::Read;

use binex::prelude::{
    Decoder, EphemerisFrame, MonumentGeoRecord, Record as BinexRecord, StreamElement,
};

/// BIN2RNX can deserialize a BINEX stream to RINEX Tokens.
pub struct BIN2RNX<'a, R: Read> {
    /// BINEX decoder
    decoder: Decoder<'a, R>,
}

impl<'a, R: Read> Iterator for BIN2RNX<'a, R> {
    type Item = Option<String>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.decoder.next() {
            Some(Ok(element)) => match element {
                StreamElement::OpenSource(msg) => match msg.record {
                    BinexRecord::MonumentGeo(geo) => for frame in geo.frames.iter() {},
                    BinexRecord::EphemerisFrame(fr) => match fr {
                        EphemerisFrame::GPS(gps) => {},
                        EphemerisFrame::GPSRaw(gps) => {},
                        EphemerisFrame::GAL(gal) => {},
                        EphemerisFrame::GLO(glo) => {},
                        EphemerisFrame::SBAS(sbas) => {},
                    },
                },
                _ => None,
            },
            Some(Err(e)) => None,
            None => None,
        }
    }
}

impl<'a, R: Read> BIN2RNX<'a, R> {
    /// Creates a new [BIN2RNX] working from [Read]able interface.
    /// It will stream Tokens as long as the interface is alive.
    pub fn new(r: R) -> Self {
        Self {
            decoder: Decoder::new(r),
        }
    }
}
