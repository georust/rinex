//! BINEX serdes oprations

use binex::prelude::Message;

mod bin2rnx;
mod rnx2bin;

pub use bin2rnx::BIN2RNX;
pub use rnx2bin::RNX2BIN;
