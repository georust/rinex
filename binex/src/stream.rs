//! BINEX Stream representation
use crate::prelude::Message;

/// [Message] [Provider]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Provider {
    /// JPL for internal needs or prototyping.
    JPL,
    /// IGS
    IGS,
    /// CU Boulder for internal needs or prototyping.
    ColoradoUnivBoulder,
    /// NRCan for internal needs or prototyping.
    NRCan,
    /// UCAR COSMIC [https://www.cosmic.ucar.edu]
    UCAR,
    /// GPS Solutions Inc.
    GPSSolutions,
    /// Astech Precision Products
    Ashtech,
    /// Topcon Positioning Systems
    Topcon,
}

impl Provider {
    /// Identify potential closed source [Provider]
    /// from parsed MID (u32)
    pub(crate) fn match_any(mid: u32) -> Option<Self> {
        if mid >= 0x80 && mid <= 0x87 {
            Some(Self::UCAR)
        } else if mid >= 0x88 && mid <= 0xa7 {
            Some(Self::Ashtech)
        } else if mid >= 0xa8 && mid <= 0xaf {
            Some(Self::Topcon)
        } else if mid >= 0xb0 && mid <= 0xb3 {
            Some(Self::GPSSolutions)
        } else if mid >= 0xb4 && mid <= 0xb7 {
            Some(Self::NRCan)
        } else if mid >= 0xb8 && mid <= 0xbf {
            Some(Self::JPL)
        } else if mid >= 0xc0 && mid <= 0xc3 {
            Some(Self::ColoradoUnivBoulder)
        } else {
            None
        }
    }
}

/// Closed source frame that we can encode but not interprate.
/// This particular [StreamElement] can be either a part of a continuous serie or self sustainable.
pub struct ClosedSourceElement<'a> {
    /// Message ID as decoded
    pub mid: u32,
    /// Provider of this frame.
    /// Only this organization may have capabilities to interprate this frame.
    pub provider: Provider,
    /// Total size of this [StreamElement] as it may be part of a continuous
    /// serie of [StreamElement]s.
    pub size: usize,
    /// Total message size (not the size of this element)
    pub mlen: usize,
    /// Raw data content that we can encode, decode but not interprate.
    pub raw: &'a [u8],
}

impl<'a> ClosedSourceElement<'a> {
    /// Interprate this [ClosedSourceElement] using custom undisclosed method.
    pub fn interprate(&self, f: &dyn Fn(&[u8])) {
        f(&self.raw[..self.size])
    }

    /// Returns reference to raw data "as is", since interpration is not possible
    pub fn raw(&self) -> &'a [u8] {
        &self.raw[..self.size]
    }
}

/// [StreamElement] represents one element of a continuous BINEX stream.
pub enum StreamElement<'a> {
    /// Open Source [Message] we can fully decode & interprate
    OpenSource(Message),
    /// One non disclosed [ClosedSourceElement] that may be part of a continuous serie of elements.
    /// Each chunk of the serie is internally limited to 4096 bytes.
    /// While we can encode and decode this serie, we cannot interprate it.
    ClosedSource(ClosedSourceElement<'a>),
}

impl<'a> From<Message> for StreamElement<'a> {
    fn from(msg: Message) -> Self {
        Self::OpenSource(msg)
    }
}

impl<'a> StreamElement<'a> {
    /// Creates a new open source [Message] ready to be encoded
    pub fn new_open_source(msg: Message) -> Self {
        Self::OpenSource(msg)
    }

    /// Creates a new self sustained closed source [StreamElement] provided by desired [Provider].
    /// ## Inputs
    /// - provider: specific [Provider]
    /// - raw: content we can encode, decode but not interprate   
    /// - size: size of this [StreamElement]
    /// - mlen: total message lenth
    pub fn new_prototype(
        provider: Provider,
        mid: u32,
        raw: &'a [u8],
        size: usize,
        mlen: usize,
    ) -> Self {
        Self::ClosedSource(ClosedSourceElement {
            raw,
            mlen,
            size,
            mid,
            provider,
        })
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::JPL].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - raw: content we can encode, decode but not interprate
    /// - size: size of the provided buffer (bytewise)
    /// - total: total size of the closed source Message (bytewise)
    pub fn jpl_prototype(raw: &'a [u8], mid: u32, size: usize, total: usize) -> Self {
        Self::new_prototype(Provider::JPL, mid, raw, size, total)
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::JPL].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - raw: content we can encode, decode but not interprate
    /// - size: size of the provided buffer (bytewise)
    /// - total: total size of the closed source Message (bytewise)
    pub fn igs_prototype(raw: &'a [u8], mid: u32, size: usize, total: usize) -> Self {
        Self::new_prototype(Provider::IGS, mid, raw, size, total)
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::ColoradoUnivBoulder].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - raw: content we can encode, decode but not interprate
    /// - size: size of the provided buffer (bytewise)
    /// - total: total size of the closed source Message (bytewise)
    pub fn cuboulder_prototype(raw: &'a [u8], mid: u32, size: usize, total: usize) -> Self {
        Self::new_prototype(Provider::ColoradoUnivBoulder, mid, raw, size, total)
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::NRCan].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - raw: content we can encode, decode but not interprate
    /// - size: size of the provided buffer (bytewise)
    /// - total: total size of the closed source Message (bytewise)
    pub fn nrcan_prototype(raw: &'a [u8], mid: u32, size: usize, total: usize) -> Self {
        Self::new_prototype(Provider::NRCan, mid, raw, size, total)
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::UCAR].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - raw: content we can encode, decode but not interprate
    /// - size: size of the provided buffer (bytewise)
    /// - total: total size of the closed source Message (bytewise)
    pub fn ucar_prototype(raw: &'a [u8], mid: u32, size: usize, total: usize) -> Self {
        Self::new_prototype(Provider::UCAR, mid, raw, size, total)
    }
}
