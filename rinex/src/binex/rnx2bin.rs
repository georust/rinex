//! RINEX to BINEX serialization
use crate::prelude::Rinex;
use binex::prelude::{Message, Meta, MonumentGeoMetadata, MonumentGeoRecord};

/// RNX2BIN can serialize a [Rinex] into a stream of BINEX [Message]s
pub struct RNX2BIN<'a> {
    /// [Rinex] that was previously collected
    rinex: &'a Rinex,
    /// State to consume the header
    state: State,
    /// BINEX [Message] encoding [Meta]
    meta: Meta,
}

fn forge_monument_geo(rinex: &Rinex) -> Option<MonumentGeoRecord> {
    let t0 = rinex.first_epoch()?;
    let mut geo = MonumentGeoRecord::default();
    geo.epoch = t0;
    geo.meta = MonumentGeoMetadata::RNX2BIN;
    geo = geo.with_software_name(&format!("geo-rust v{}", env!("CARGO_PKG_VERSION")));
    Some(geo)
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum State {
    /// Describes the RINEX format, Constellation and revision
    #[default]
    HeaderPkgVersion,
    MonumentGeo,
    AnnounceHeaderComments,
    HeaderComments,
    AnnounceRecord,
    Record,
}

impl<'a> Iterator for RNX2BIN<'a> {
    type Item = Message;
    /// Consume [Rinex] into [Message] stream
    fn next(&mut self) -> Option<Self::Item> {
        let content = match self.state {
            State::HeaderPkgVersion => {
                let mut geo = forge_monument_geo(&self.rinex)?;
                geo.comments
                    .push(format!("RNX2BIN from {}", self.rinex.header.rinex_type));
                geo.comments.push("STREAM starting!".to_string());
                self.state = State::MonumentGeo;
                Some(geo)
            },
            State::MonumentGeo => {
                let mut geo = forge_monument_geo(&self.rinex)?;
                // TODO
                // Geo::OperatorName
                // Geo::ObserverName
                // Geo::AgencyName
                // Geo::MonumentName
                // Geo::MonumentNumber
                // Geo::MarkerName
                // Geo::MarkerNumber
                // Geo::ReferenceName
                // Geo::ReferenceNumber(=DOMES)
                // Geo::ReferenceDate?
                if !self.rinex.header.comments.is_empty() {
                    self.state = State::AnnounceHeaderComments;
                } else {
                    self.state = State::AnnounceRecord;
                }
                Some(geo)
            },
            State::AnnounceHeaderComments => {
                let mut geo = forge_monument_geo(&self.rinex)?;
                geo = geo.with_comment("RINEX Header comments following!");
                self.state = State::HeaderComments;
                Some(geo)
            },
            State::HeaderComments => {
                let mut geo = forge_monument_geo(&self.rinex)?;
                for comment in self.rinex.header.comments.iter() {
                    geo = geo.with_comment(comment);
                }
                self.state = State::AnnounceRecord;
                Some(geo)
            },
            State::AnnounceRecord => {
                let mut geo = forge_monument_geo(&self.rinex)?;
                geo = geo.with_comment("RINEX RECORD starting!");
                self.state = State::Record;
                Some(geo)
            },
            State::Record => {
                // not available yet
                // NAV RINEX => Ephemeris BINEX
                // METEO RINEX => Meteo/GEO/PVT BINEX
                // OBS RINEX => OBS BINEX
                // IONEX?
                // CLOCK?
                None
            },
        };

        if let Some(content) = content {
            // forge new message
            Some(Message {
                meta: self.meta,
                record: content.into(),
            })
        } else {
            None
        }
    }
}

impl Rinex {
    /// Build a new [RNX2BIN] to serialize this [Rinex]
    /// into a BINEX stream.
    /// ## Inputs:
    /// - meta: BINEX encoding [Meta]
    /// ## Output
    /// - [RNX2BIN]: a BINEX [Message] Iterator
    ///
    /// This currently applies to Observation and Navigation RINEX.
    /// Eventually, this should work well on any temporal RINEX,
    /// that means all format but ANTEX.
    /// ```
    /// let rinex = Rinex::from_file(
    ///     "../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx"
    ///     ).unwrap();
    ///
    /// let mut buf = [0; 1024];
    /// let mut streamer = rinex.rnx2bin();
    ///
    /// while let Some(msg) = streamer.next() {
    ///     // usually you want to dump this message
    ///     // and then stream to a writable I/O interface.
    ///     // To do so, use the encode method and a temporary buffer:
    ///     let size = msg.encode(&mut buf).unwrap();
    ///     // send!
    /// }
    /// ```
    pub fn rnx2bin<'a>(&'a self, meta: Meta) -> RNX2BIN<'a> {
        RNX2BIN {
            meta,
            rinex: self,
            state: State::default(),
        }
    }
}
