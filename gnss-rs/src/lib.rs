#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docrs, feature(doc_cfg))]

#[macro_use]
mod macros;

mod snr;
mod code;
pub mod sv;
pub mod constellation;

use constellation::Constellation;

pub mod prelude {
    pub use crate::sv::SV;
    pub use crate::snr::SNR;
    pub use crate::code::Code;
    pub use crate::constellation::Constellation;
}

mod sbas;

#[cfg(feature = "sbas")]
pub use sbas::sbas_selection;
