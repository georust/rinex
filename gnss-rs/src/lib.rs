#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docrs, feature(doc_cfg))]

#[macro_use]
mod macros;

pub mod sv;

pub mod constellation;
use constellation::Constellation;

pub mod prelude {
    pub use crate::constellation::Constellation;
    pub use crate::sv::SV;
}

mod sbas;

#[cfg(feature = "sbas")]
pub use sbas::sbas_selection;
