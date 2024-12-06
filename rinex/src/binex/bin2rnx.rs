//! BINEX to RINEX deserialization
use std::io::Read;

use crate::prelude::{Duration, Epoch, Rinex};

use binex::prelude::{
    Decoder, EphemerisFrame, MonumentGeoRecord, Record as BinexRecord, StreamElement,
};

/// [SnapshotMode] controls and determines the
/// RINEX file production coming from a BINEX stream.
/// A BINEX stream being potentially infinite (as long as
/// production hardware or data source exists on the other end)
#[derive(Debug, Copy, PartialEq, Clone)]
pub enum SnapshotMode {
    /// Dump as RINEX every day at midnight.
    /// This is the prefered [SnapshotMode] because
    /// standard RINEX span 24h.
    DailyMidnight,
    /// Dump as RINEX every day at midnight.
    /// Use this to produce 12h RINEX.
    DailyMidnightNoon,
    /// Dump as RINEX every hour.
    /// This is used when producing Hourly RINEX.
    Hourly,
    /// Standard RINEX files span 24h.
    /// Dump as RINEX periodically
    Period(Duration),
}

/// [Postponing] offers several options to postpone the BINEX message collection.
/// It allows to accurately control when the stream listener picks up the
/// BINEX content that should be collected.
#[derive(Debug, Copy, PartialEq, Clone)]
pub enum Postponing {
    /// RINEX collection starts on first valid BINEX byte
    None,
    /// RINEX collection will start once system time reaches
    /// this value. Note that system time access is OS dependent.
    SystemTime(Epoch),
    /// RINEX collection starts after `size` BINEX bytes have been collected
    Size(usize),
    /// RINEX collection starts after discarding `size` valid BINEX messages
    Messages(usize),
}

/// BIN2RNX is a RINEX producer from a BINEX stream.
/// It can serialize the streamed messages and collect them as RINEX.
/// The production behavior is defined by [SnapshotMode]
pub struct BIN2RNX<'a, R: Read> {
    /// Deploy time
    deploy_t: Epoch,
    /// True when collecting
    pub collecting: bool,
    /// Snapshot mode
    pub snapshot_mode: SnapshotMode,
    /// Postponing option
    pub postponing: Postponing,
    /// BINEX decoder working on [Read]able stream
    decoder: Decoder<'a, R>,
}

impl<'a, R: Read> Iterator for BIN2RNX<'a, R> {
    type Item = Option<String>;
    fn next(&mut self) -> Option<Self::Item> {
        // match self.decoder.next() {
        //     Some(Ok(element)) => match element {
        //         StreamElement::OpenSource(msg) => match msg.record {
        //             BinexRecord::MonumentGeo(geo) => for frame in geo.frames.iter() {},
        //             BinexRecord::EphemerisFrame(fr) => match fr {
        //                 EphemerisFrame::GPS(gps) => {},
        //                 EphemerisFrame::GPSRaw(gps) => {},
        //                 EphemerisFrame::GAL(gal) => {},
        //                 EphemerisFrame::GLO(glo) => {},
        //                 EphemerisFrame::SBAS(sbas) => {},
        //             },
        //         },
        //         _ => None,
        //     },
        //     Some(Err(e)) => None,
        //     None => None,
        // }
        None
    }
}

impl<'a, R: Read> BIN2RNX<'a, R> {
    /// Creates a new [BIN2RNX] working from [Read]able interface.
    /// It will stream Tokens as long as the interface is alive.
    /// NB:
    /// - [BIN2RNX] needs the system time to be determined for the postponing
    /// mechanism. If determination fails, this method will panic.
    /// We propose [Self::new_system_time] if you want to manually
    /// define "now".
    /// - since RINEX is fully open source, only open source BINEX messages
    /// may be picked up and collected: closed source elements are discarded.
    pub fn new(snapshot_mode: SnapshotMode, postponing: Postponing, read: R) -> Self {
        Self::new_system_time(
            Epoch::now().unwrap_or_else(|e| panic!("system time determination failed with {}", e)),
            snapshot_mode,
            postponing,
            read,
        )
    }

    /// Infaillible [BIN2RNX] creation, use this if you have no means to access system time.
    /// Define it yourself with "now". Refer to [Self::new] for more information.
    pub fn new_system_time(
        now: Epoch,
        snapshot_mode: SnapshotMode,
        postponing: Postponing,
        read: R,
    ) -> Self {
        Self {
            postponing,
            deploy_t: now,
            snapshot_mode,
            decoder: Decoder::new(read),
            collecting: postponing == Postponing::None,
        }
    }

    /// Creates a new [BIN2RNX] that will collect a [Rinex] daily
    /// with possible postponing
    pub fn new_daily_midnight(postponing: Postponing, read: R) -> Self {
        Self::new(SnapshotMode::DailyMidnight, postponing, read)
    }

    /// Creates a new [BIN2RNX] that will collect a [Rinex] twice a day
    /// with possible postponing
    pub fn new_midnight_noon(postponing: Postponing, read: R) -> Self {
        Self::new(SnapshotMode::DailyMidnightNoon, postponing, read)
    }

    /// Creates a new [BIN2RNX] that will collect a [Rinex] twice a day
    /// with possible postponing
    pub fn new_hourly(postponing: Postponing, read: R) -> Self {
        Self::new(SnapshotMode::Hourly, postponing, read)
    }

    /// Creates a new [BIN2RNX] that will collect a [Rinex] periodically,
    /// with possible postponing
    pub fn new_periodic(period: Duration, postponing: Postponing, read: R) -> Self {
        Self::new(SnapshotMode::Period(period), postponing, read)
    }
}
