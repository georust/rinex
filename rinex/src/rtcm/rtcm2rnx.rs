//! RTCM to RINEX deserialization
use std::io::Read;

use crate::prelude::Rinex;

/// RTCM2RNX can deserialize a RTCM stream to RINEX Tokens.
pub struct RTCM2RNX<R: Read> {
    /// RTCMEX decoder
    decoder: Decoder<R>,
}

impl Iterator for RTCM2RNX {
    type Item = Option<String>;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl RTCM2RNX {
    pub fn new<R: Read>(r: R) -> Self {
        Self {
            decoder: Decoder::new(r),
        }
    }
}
