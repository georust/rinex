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

/// Closed source frame that we can encode but not interprate.
/// This particular [StreamElement] can be either a part of a continuous serie or self sustainable.
pub struct ClosedSourceElement<'a> {
    /// Provider of this frame.
    /// Only this organization may have capabilities to interprate this frame.
    pub provider: Provider,
    /// Size of this element. Use this to determine the packet index
    /// in a continuous stream of undisclosed [StreamElement]s.
    pub size: usize,
    /// Total size of this undisclosed message. Use this to determine the packet index
    /// in a continuous stream of undisclosed [StreamElement]s.
    pub total: usize,
    /// Raw data content that we can encode, decode but not interprate.
    raw: &'a [u8],
}

impl<'a> ClosedSourceElement<'a> {
    /// Interprate this [ClosedSourceElement] using custom undisclosed method.
    /// ```
    ///
    /// ```
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
    /// - total: total size of the [StreamElement] serie
    pub fn new_prototype(provider: Provider, raw: &'a [u8], size: usize, total: usize) -> Self {
        Self::ClosedSource(ClosedSourceElement {
            raw,
            total,
            size,
            provider,
        })
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::JPL].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - raw: content we can encode, decode but not interprate
    /// - size: size of the provided buffer (bytewise)
    /// - total: total size of the closed source Message (bytewise)
    pub fn jpl_prototype(raw: &'a [u8], size: usize, total: usize) -> Self {
        Self::new_prototype(Provider::JPL, raw, size, total)
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::JPL].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - raw: content we can encode, decode but not interprate
    /// - size: size of the provided buffer (bytewise)
    /// - total: total size of the closed source Message (bytewise)
    pub fn igs_prototype(raw: &'a [u8], size: usize, total: usize) -> Self {
        Self::new_prototype(Provider::IGS, raw, size, total)
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::ColoradoUnivBoulder].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - raw: content we can encode, decode but not interprate
    /// - size: size of the provided buffer (bytewise)
    /// - total: total size of the closed source Message (bytewise)
    pub fn cuboulder_prototype(raw: &'a [u8], size: usize, total: usize) -> Self {
        Self::new_prototype(Provider::ColoradoUnivBoulder, raw, size, total)
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::NRCan].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - raw: content we can encode, decode but not interprate
    /// - size: size of the provided buffer (bytewise)
    /// - total: total size of the closed source Message (bytewise)
    pub fn nrcan_prototype(raw: &'a [u8], size: usize, total: usize) -> Self {
        Self::new_prototype(Provider::NRCan, raw, size, total)
    }

    /// Add one closed source [StreamElement]s provided by desired [Provider::UCAR].
    /// While we can encode this into a BINEX stream, only this organization can fully interprate the resulting stream.
    /// ## Inputs
    /// - raw: content we can encode, decode but not interprate
    /// - size: size of the provided buffer (bytewise)
    /// - total: total size of the closed source Message (bytewise)
    pub fn ucar_prototype(raw: &'a [u8], size: usize, total: usize) -> Self {
        Self::new_prototype(Provider::UCAR, raw, size, total)
    }
}
