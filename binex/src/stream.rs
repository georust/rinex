//! BINEX Stream representation
use crate::prelude::{ClosedSourceMeta, Message, Meta};

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
    /// UCAR COSMIC <https://www.cosmic.ucar.edu>
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
        if (0x80..=0x87).contains(&mid) {
            Some(Self::UCAR)
        } else if (0x88..=0xa7).contains(&mid) {
            Some(Self::Ashtech)
        } else if (0xa8..=0xaf).contains(&mid) {
            Some(Self::Topcon)
        } else if (0xb0..=0xb3).contains(&mid) {
            Some(Self::GPSSolutions)
        } else if (0xb4..=0xb7).contains(&mid) {
            Some(Self::NRCan)
        } else if (0xb8..=0xbf).contains(&mid) {
            Some(Self::JPL)
        } else if (0xc0..=0xc3).contains(&mid) {
            Some(Self::ColoradoUnivBoulder)
        } else {
            None
        }
    }
}

/// Closed source frame that we can encode but not interprate.
/// This particular [StreamElement] can be either a part of a continuous serie or self sustainable.
pub struct ClosedSourceElement<'a> {
    /// [ClosedSourceMeta]
    pub closed_meta: ClosedSourceMeta,
    /// Raw data starting at first byte of undisclosed payload.
    pub raw: &'a [u8],
}

impl<'a> ClosedSourceElement<'a> {
    /// Interprate this [ClosedSourceElement] using custom undisclosed method.
    pub fn interprate(&self, f: &dyn Fn(&[u8])) {
        f(&self.raw[..self.closed_meta.size])
    }

    /// Returns reference to raw data "as is", since interpration is not possible
    pub fn raw(&self) -> &'a [u8] {
        &self.raw[..self.closed_meta.size]
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
    /// - meta: [Meta] data of this prototype
    /// - provider: specific [Provider]
    /// - mid: message ID
    /// - mlen: total payload length (bytes)
    /// - raw: chunk we can encode, decode but not fully interprate   
    /// - size: size of this chunk
    pub fn new_prototype(
        open_meta: Meta,
        provider: Provider,
        mid: u32,
        mlen: usize,
        raw: &'a [u8],
        size: usize,
    ) -> Self {
        Self::ClosedSource(ClosedSourceElement {
            raw,
            closed_meta: ClosedSourceMeta {
                mid,
                mlen,
                open_meta,
                provider,
                offset: 0,
                size,
            },
        })
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::JPL].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - meta: [Meta] of this message prototype
    /// - mid: message ID
    /// - mlen: total payload length (bytes)
    /// - raw: content we can encode, decode but not interprate
    /// - total: total size of the closed source Message (bytewise).
    /// It's either equal to [Meta::mlen] if this prototype if self sustainable,
    /// or larger, in case this prototype is only one element of a serie.
    pub fn jpl_prototype(meta: Meta, mid: u32, mlen: usize, raw: &'a [u8], size: usize) -> Self {
        Self::new_prototype(meta, Provider::JPL, mid, mlen, raw, size)
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::JPL].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - meta: [Meta] of this message prototype
    /// - raw: content we can encode, decode but not interprate
    /// - total: total size of the closed source Message (bytewise).
    /// It's either equal to [Meta::mlen] if this prototype if self sustainable,
    /// or larger, in case this prototype is only one element of a serie.
    pub fn igs_prototype(meta: Meta, mid: u32, mlen: usize, raw: &'a [u8], size: usize) -> Self {
        Self::new_prototype(meta, Provider::IGS, mid, mlen, raw, size)
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::ColoradoUnivBoulder].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - meta: [Meta] of this message prototype
    /// - mid: message ID
    /// - mlen: total payload length (bytes)
    /// - raw: content we can encode, decode but not interprate
    /// - total: total size of the closed source Message (bytewise).
    /// It's either equal to [Meta::mlen] if this prototype if self sustainable,
    /// or larger, in case this prototype is only one element of a serie.
    pub fn cuboulder_prototype(
        meta: Meta,
        mid: u32,
        mlen: usize,
        raw: &'a [u8],
        size: usize,
    ) -> Self {
        Self::new_prototype(meta, Provider::ColoradoUnivBoulder, mid, mlen, raw, size)
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::NRCan].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - meta: [Meta] of this message prototype
    /// - mid: message ID
    /// - mlen: total payload length (bytes)
    /// - raw: content we can encode, decode but not interprate
    /// - total: total size of the closed source Message (bytewise).
    /// It's either equal to [Meta::mlen] if this prototype if self sustainable,
    /// or larger, in case this prototype is only one element of a serie.
    pub fn nrcan_prototype(meta: Meta, mid: u32, mlen: usize, raw: &'a [u8], size: usize) -> Self {
        Self::new_prototype(meta, Provider::NRCan, mid, mlen, raw, size)
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::UCAR].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - meta: [Meta] of this message prototype
    /// - mid: message ID
    /// - mlen: total payload length (bytes)
    /// - raw: content we can encode, decode but not interprate
    /// - total: total size of the closed source Message (bytewise).
    /// It's either equal to [Meta::mlen] if this prototype if self sustainable,
    /// or larger, in case this prototype is only one element of a serie.
    pub fn ucar_prototype(meta: Meta, mid: u32, mlen: usize, raw: &'a [u8], size: usize) -> Self {
        Self::new_prototype(meta, Provider::UCAR, mid, mlen, raw, size)
    }
}
