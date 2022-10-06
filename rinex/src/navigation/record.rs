//! `NavigationData` parser and related methods
use thiserror::Error;
use std::str::FromStr;
use strum_macros::EnumString;
use std::collections::{BTreeMap, HashMap};

use crate::epoch;
use crate::header;
use crate::sv;
use crate::sv::Sv;
use crate::epoch::{Epoch, ParseDateError};
use crate::version::Version;
use crate::constellation::Constellation;

use crate::navigation::{
    ionmessage, 
    eopmessage,
    database, database::{NAV_MESSAGES, DbItem, DbItemError},
};

use super::{
    EopMessage, 
    StoMessage, 
    IonMessage,
    KbModel,
    BdModel,
    NgModel,
};

use std::io::Write;
use crate::writer::BufferedWriter;

/// Possible Navigation Frame declinations for an epoch
#[derive(Debug, Copy, Clone)]
#[derive(PartialEq, PartialOrd)]
#[derive(Eq, Ord)]
#[derive(EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum FrameClass {
    #[strum(serialize = "EPH")]
    Ephemeris,
    #[strum(serialize = "STO")]
    SystemTimeOffset,
    #[strum(serialize = "EOP")]
    EarthOrientation,
    #[strum(serialize = "ION")]
    IonosphericModel,
}

impl Default for FrameClass {
    fn default() -> Self{
        Self::Ephemeris
    }
}

impl std::fmt::Display for FrameClass {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Ephemeris => f.write_str("EPH"),
            Self::SystemTimeOffset => f.write_str("STO"),
            Self::EarthOrientation => f.write_str("EOP"),
            Self::IonosphericModel => f.write_str("ION"),
        }
    }
}

/// Navigation Message Types 
#[derive(Debug, Copy, Clone)]
#[derive(PartialEq, PartialOrd)]
#[derive(Eq, Ord)]
#[derive(EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum MsgType {
    /// Legacy NAV
    LNAV,
    /// FDMA
    FDMA,
    /// IFNV,
    IFNV,
    /// D1
    D1,
    /// D2
    D2,
    /// D1D2
    D1D2,
    /// SBAS
    SBAS,
    /// CNVX special marker
    CNVX,
}

impl Default for MsgType {
    fn default() -> Self {
        Self::LNAV
    }
}

impl std::fmt::Display for MsgType {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::LNAV => f.write_str("LNAV"),
            Self::FDMA => f.write_str("FDMA"),
            Self::IFNV => f.write_str("IFNV"),
            Self::D1 => f.write_str("D1"),
            Self::D2 => f.write_str("D2"),
            Self::D1D2 => f.write_str("D1D2"),
            Self::SBAS => f.write_str("SBAS"),
            Self::CNVX => f.write_str("CNVX"),
        }
    }
}

/// Navigation Frame for a given epoch
#[derive(Debug, Clone)]
#[derive(PartialEq)]
#[derive(EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Frame {
    /// Ephemeris for a given Vehicule `Sv`,
    /// with vehicule internal clock bias, clock drift and clock drift rate.
    /// Rest of data is constellation dependent, see
    /// RINEX specifications or db/NAV/navigation.json.
    Eph(MsgType, Sv, f64, f64, f64, HashMap<String, DbItem>),
    /// Earth Orientation Parameters message 
    Eop(EopMessage),
    /// Ionospheric Model Message
    Ion(Sv, IonMessage),
    /// System Time Offset Message
    Sto(Sv, StoMessage),
}

impl Frame {
    /// Unwraps self as Ephemeris frame
    pub fn as_eph (&self) -> Option<(MsgType, Sv, f64, f64, f64, &HashMap<String, DbItem>)> {
        match self {
            Self::Eph(msg, sv, clk, clk_dr, clk_drr, map) => Some((*msg, *sv, *clk, *clk_dr, *clk_drr, map)),
            _ => None,
        }
    }
    /// Unwraps self as mutable Ephemeris frame reference
    pub fn as_mut_eph (&mut self) -> Option<(&mut MsgType, &mut Sv, &mut f64, &mut f64, &mut f64, &mut HashMap<String, DbItem>)> {
        match self {
            Self::Eph(msg, sv, clk, clk_dr, clk_drr, map) => Some((msg, sv, clk, clk_dr, clk_drr, map)),
            _ => None,
        }
    }
    /// Unwraps self as Ionospheric Model frame
    pub fn as_ion (&self) -> Option<(&Sv, &IonMessage)> {
        match self {
            Self::Ion(sv, fr) => Some((sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self as mutable Ionospheric Model frame reference
    pub fn as_mut_ion (&mut self) -> Option<(&mut Sv, &mut IonMessage)> {
        match self {
            Self::Ion(sv, fr) => Some((sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self as Earth Orientation frame
    pub fn as_eop (&self) -> Option<&EopMessage> {
        match self {
            Self::Eop(fr) => Some(fr),
            _ => None,
        }
    }
    /// Unwraps self as Mutable Earth Orientation frame reference
    pub fn as_mut_eop (&mut self) -> Option<&mut EopMessage> {
        match self {
            Self::Eop(fr) => Some(fr),
            _ => None,
        }
    }
    /// Unwraps self as System Time Offset frame
    pub fn as_sto (&self) -> Option<(&Sv, &StoMessage)> {
        match self {
            Self::Sto(sv, fr) => Some((sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self as mutable System Time Offset frame reference
    pub fn as_mut_sto (&mut self) -> Option<(&mut Sv, &mut StoMessage)> {
        match self {
            Self::Sto(sv, fr) => Some((sv, fr)),
            _ => None,
        }
    }
}

/// Navigation Record.
/// Data is sorted by epoch, and by Frame class.
pub type Record = BTreeMap<Epoch, BTreeMap<FrameClass, Vec<Frame>>>;

/// Returns true if given content matches the beginning of a 
/// Navigation record epoch
pub fn is_new_epoch (line: &str, v: Version) -> bool {
    if v.major < 3 { // old RINEX
        if line.len() < 23 {
            return false // not enough bytes 
                // to describe a PRN and an Epoch
        }
        let (prn, _) = line.split_at(2);
        // 1st entry is a valid integer number
        if u8::from_str_radix(prn.trim(), 10).is_err() {
            return false
        }
        // rest matches a valid epoch descriptor
        let datestr = &line[3..22];
        epoch::str2date(&datestr).is_ok()

    } else if v.major == 3 { // RINEX V3
        if line.len() < 24 {
            return false // not enough bytes
                // to describe an SVN and an Epoch
        }
        // 1st entry matches a valid SV description
        let (sv, _) = line.split_at(4);
        if Sv::from_str(sv).is_err() {
            return false
        }
        // rest matches a valid epoch descriptor
        let datestr = &line[4..23];
        epoch::str2date(&datestr).is_ok()

    } else { // Modern --> easy 
        if let Some(c) = line.chars().nth(0) {
            c == '>' // new epoch marker 
        } else {
            false
        }
    }
}

/// Navigation Record Parsing Error
#[derive(Error, Debug)]
pub enum Error {
    #[error("epoch is missing data")]
    MissingData,
    #[error("file operation error")]
    FileIoError(#[from] std::io::Error),
    #[error("failed to locate revision in db")]
    DataBaseRevisionError,
    #[error("failed to parse msg type")]
    SvError(#[from] sv::Error),
    #[error("failed to parse cplx data")]
    ParseComplexError(#[from] DbItemError),
    #[error("failed to parse sv::prn")]
    ParseIntError(#[from] std::num::ParseIntError), 
    #[error("failed to parse sv clock fields")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse epoch date")]
    ParseDateError(#[from] ParseDateError),
    #[error("failed to identify class/type")]
    StrumError(#[from] strum::ParseError), 
    #[error("failed to parse ION message")]
    IonMessageError(#[from] ionmessage::Error),
    #[error("failed to parse EOP message")]
    EopMessageError(#[from] eopmessage::Error),
}

/// Builds `Record` entry for `NavigationData`
pub fn parse_epoch (version: Version, constell: Constellation, content: &str) ->
        Result<(Epoch, FrameClass, Frame), Error>
{
    if content.starts_with(">") {
        build_modern_record_entry(content)
    } else {
        build_v2_v3_record_entry(version, constell, content)
    }
}

/// Builds `Record` entry for Modern NAV frames
fn build_modern_record_entry (content: &str) ->
        Result<(Epoch, FrameClass, Frame), Error>
{
    let mut lines = content.lines();
    let line = match lines.next() {
        Some(l) => l,
        _ => return Err(Error::MissingData),
    };

    let (_, rem) = line.split_at(2);
    let (frame_class, rem) = rem.split_at(4);
    let (svnn, rem) = rem.split_at(4);

    let frame_class = FrameClass::from_str(frame_class.trim())?;
    let sv = Sv::from_str(svnn.trim())?;
    let msg_type = MsgType::from_str(rem.trim())?;

    let (epoch, fr): (Epoch, Frame) = match frame_class {
        FrameClass::Ephemeris => {
            let line = match lines.next() {
                Some(l) => l,
                _ => return Err(Error::MissingData),
            };
            
            let (svnn, rem) = line.split_at(4);
            let sv = Sv::from_str(svnn.trim())?;
            let (epoch, rem) = rem.split_at(20);
            let epoch = Epoch {
                date: epoch::str2date(epoch.trim())?,
                flag: epoch::EpochFlag::Ok,
            };

            let (clk_bias, rem) = rem.split_at(19);
            let (clk_dr, clk_drr) = rem.split_at(19);
            let clk = f64::from_str(clk_bias.replace("D","E").trim())?;
            let clk_dr = f64::from_str(clk_dr.replace("D","E").trim())?;
            let clk_drr = f64::from_str(clk_drr.replace("D","E").trim())?;
            let map = parse_complex_map(
                Version { major: 4, minor: 0 },
                sv.constellation,
                lines)?;
            (epoch, Frame::Eph(msg_type, sv, clk, clk_dr, clk_drr, map))
        },
        FrameClass::SystemTimeOffset => {
            let line = match lines.next() {
                Some(l) => l,
                _ => return Err(Error::MissingData),
            };
            
            let (epoch, rem) = line.split_at(23);
            let (system, _) = rem.split_at(5);
            let epoch = Epoch {
                date: epoch::str2date(epoch.trim())?,
                flag: epoch::EpochFlag::Ok,
            };
            let line = match lines.next() {
                Some(l) => l,
                _ => return Err(Error::MissingData),
            };
            let (time, rem) = line.split_at(23);
            let (a0, rem) = rem.split_at(19);
            let (a1, rem) = rem.split_at(19);
            let (a2, rem) = rem.split_at(19);

            let t_tm = f64::from_str(time.trim())?;
            let msg = StoMessage {
                system: system.trim().to_string(),
                t_tm: t_tm as u32,
                a: (
                    f64::from_str(a0.trim()).unwrap_or(0.0_f64),
                    f64::from_str(a1.trim()).unwrap_or(0.0_f64),
                    f64::from_str(a2.trim()).unwrap_or(0.0_f64),
                ),
                utc: rem.trim().to_string(),
            };
            (epoch, Frame::Sto(sv, msg))
        },
        FrameClass::EarthOrientation => {
            let (epoch, msg) = EopMessage::parse(lines)?;
            (epoch, Frame::Eop(msg))
        },
        FrameClass::IonosphericModel => {
            let (epoch, msg): (Epoch, IonMessage) = match msg_type {
                MsgType::IFNV => {
                    let (epoch, sv, model) = NgModel::parse(lines)?;
                    (epoch, IonMessage::NequickGModel(model))
                },
                MsgType::CNVX => {
                    match sv.constellation {
                        Constellation::BeiDou => {
                            let (epoch, sv, model) = BdModel::parse(lines)?;
                            (epoch, IonMessage::BdgimModel(model))
                        },
                        _ => {
                            let (epoch, sv, model) = KbModel::parse(lines)?;
                            (epoch, IonMessage::KlobucharModel(model))
                        },
                    }
                },
                _ => {
                    let (epoch, sv, model) = KbModel::parse(lines)?;
                    (epoch, IonMessage::KlobucharModel(model))
                }
            };
            (epoch, Frame::Ion(msg))
        },
    };
    Ok((epoch, frame_class, fr))
}

/// Builds `Record` entry for Old NAV frames
fn build_v2_v3_record_entry (version: Version, constell: Constellation, content: &str) ->
        Result<(Epoch, FrameClass, Frame), Error>
{
    let mut lines = content.lines();
    let line = match lines.next() {
        Some(l) => l,
        _ => return Err(Error::MissingData), 
    };
    
    let svnn_offset: usize = match version.major {
        1|2 => 2, // Y
        3 => 4, // XYY
        _ => unreachable!(),
    };

    let (svnn, rem) = line.split_at(svnn_offset);
    let (date, rem) = rem.split_at(20);
    let (clk_bias, rem) = rem.split_at(19);
    let (clk_dr, clk_drr) = rem.split_at(19);

    let sv : Sv = match version.major {
        1|2 => {
            match constell {
                Constellation::Mixed => { // not sure that even exists
                    Sv::from_str(svnn.trim())?
                },
                _ => {
                    Sv {
                        constellation: constell.clone(),
                        prn: u8::from_str_radix(svnn.trim(), 10)?,
                    }
                },
            }
        },
        3 => Sv::from_str(svnn.trim())?,
        _ => unreachable!(),
    };

    let clk = f64::from_str(clk_bias.replace("D","E").trim())?;
    let clk_dr = f64::from_str(clk_dr.replace("D","E").trim())?;
    let clk_drr = f64::from_str(clk_drr.replace("D","E").trim())?;
    let map = parse_complex_map(version, sv.constellation, lines)?;
    let fr = Frame::Eph(MsgType::LNAV, sv, clk, clk_dr, clk_drr, map); // indicate legacy frame
    Ok((
        Epoch::new(
            epoch::str2date(date)?,
            epoch::EpochFlag::default(), // flag never given in NAV 
        ),
        FrameClass::Ephemeris, // legacy: Only Ephemeris exist
        fr, // ephemeris frame
    ))
}

/// Parses constellation + revision dependent complex map 
fn parse_complex_map (version: Version, constell: Constellation, mut lines: std::str::Lines<'_>) 
        -> Result<HashMap<String, DbItem>, Error>
{
    // locate closest revision in db
    let db_revision = match database::closest_revision(constell, version) {
        Some(v) => v,
        _ => return Err(Error::DataBaseRevisionError),
    };

    // retrieve db items / fields to parse
    let items :Vec<_> = NAV_MESSAGES
        .iter()
        .filter(|r| r.constellation == constell.to_3_letter_code())
        .map(|r| {
            r.revisions
                .iter()
                .filter(|r| // identified db revision
                    u8::from_str_radix(r.major, 10).unwrap() == db_revision.major
                    && u8::from_str_radix(r.minor, 10).unwrap() == db_revision.minor
                )
                .map(|r| &r.items)
                .flatten()
        })
        .flatten()
        .collect();

    // parse items
    let mut line = match lines.next() {
        Some(l) => l,
        _ => return Err(Error::MissingData),
    };
    let mut new_line = true;
    let mut total :usize = 0;
    let mut map :HashMap<String, DbItem> = HashMap::new();
    for item in items.iter() {
        let (k, v) = item;
        let offset :usize = match new_line {
            false => 19,
            true => {
                new_line = false;
                if version.major == 3 {
                    22+1
                } else {
                    22
                }
            },
        };
        if line.len() >= 19 { // handle empty fields, that might exist..
            let (content, rem) = line.split_at(offset);
            total += offset;
            line = rem.clone();

            if !k.contains(&"spare") { // --> got something to parse in db
                if let Ok(cplx) = DbItem::new(v, content.trim(), constell) {
                    map.insert(k.to_string(), cplx);
                }
            }

            if total >= 76 {
                new_line = true;
                total = 0;
                if let Some(l) = lines.next() {
                    line = l;
                } else {
                    break
                }
            }
        } else { // early EOL (blank)
            total = 0;
            new_line = true;
            if let Some(l) = lines.next() {
                line = l
            } else {
                break
            }
        }
    }
    Ok(map)
}

/// Writes given epoch into stream 
pub fn write_epoch (
        epoch: &Epoch, 
        data: &BTreeMap<FrameClass, Vec<Frame>>,
        header: &header::Header,
        writer: &mut BufferedWriter,
    ) -> Result<(), Error> {
    if header.version.major < 4 {
        write_epoch_v2_v3(epoch, data, header, writer)
    } else {
        write_epoch_v4(epoch, data, header, writer)
    }
}

fn write_epoch_v2_v3 (
        epoch: &Epoch, 
        data: &BTreeMap<FrameClass, Vec<Frame>>,
        header: &header::Header,
        writer: &mut BufferedWriter,
    ) -> Result<(), Error> {
    for (class, frames) in data.iter() {
        if *class == FrameClass::Ephemeris {
            for frame in frames.iter() {
                let (_, sv, clk_off, clk_dr, clk_drr, data) = frame.as_eph()
                    .unwrap();
                match &header.constellation {
                    Some(Constellation::Mixed) => {
                        // Mixed constellation context
                        // we need to fully describe the vehicule
                        write!(writer, "{} ", sv)?;
                    },
                    Some(_) => {
                        // Unique constellation context:
                        // in V2 format, only PRN is shown
                        write!(writer, "{:2} ", sv.prn)?;
                    },
                    None => panic!("producing data with no constellation previously defined"),
                }
                if header.version.major < 3 {
                    write!(writer, "{} ", epoch.to_string_nav_v2())?;
                } else {
                    write!(writer, "{} ", epoch.to_string_nav_v3())?;
                }
                write!(writer,
                    "{:14.13E} {:14.13E} {:14.13E}\n     ",
                    clk_off,
                    clk_dr,
                    clk_drr)?;
                // locate closest revision in db
                let db_revision = match database::closest_revision(sv.constellation, header.version) {
                    Some(v) => v,
                    _ => return Err(Error::DataBaseRevisionError),
                };
                // retrieve db items / fields to generate,
                // for this revision
                let items: Vec<_> = NAV_MESSAGES
                    .iter()
                    .filter(|r| r.constellation == sv.constellation.to_3_letter_code())
                    .map(|r| {
                        r.revisions
                            .iter()
                            .filter(|r| // identified db revision
                                u8::from_str_radix(r.major, 10).unwrap() == db_revision.major
                                && u8::from_str_radix(r.minor, 10).unwrap() == db_revision.minor
                            )
                            .map(|r| &r.items)
                            .flatten()
                    })
                    .flatten()
                    .collect();
                let nb_items_per_line = 4;
                let mut chunks = items
                    .chunks_exact(nb_items_per_line)
                    .peekable();
                while let Some(chunk) = chunks.next() {
                    if chunks.peek().is_some() {
                        for (key, _) in chunk {
                            if let Some(data) = data.get(*key) {
                                write!(writer, "{} ", data.to_string())?
                            } else {
                                write!(writer, "              ")?;
                            }
                        }
                        write!(writer, "\n     ")?;
                    } else {
                        // last row
                        for (key, _) in chunk {
                            if let Some(data) = data.get(*key) {
                                write!(writer, "{} ", data.to_string())?
                            } else {
                                write!(writer, "              ")?;
                            }
                        }
                        write!(writer, "\n")?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn write_epoch_v4 (
        epoch: &Epoch, 
        data: &BTreeMap<FrameClass, Vec<Frame>>,
        header: &header::Header,
        writer: &mut BufferedWriter,
    ) -> Result<(), Error> {
    for (class, frames) in data.iter() {
        if *class == FrameClass::Ephemeris {
            for frame in frames.iter() {
                write!(writer, "> {}", class)?;
                let (msgtype, sv, clk_off, clk_dr, clk_drr, data) = frame.as_eph()
                    .unwrap();
                write!(writer, "{} {}\n", sv, msgtype)?;
                match &header.constellation {
                    Some(Constellation::Mixed) => {
                        // Mixed constellation context
                        // we need to fully describe the vehicule
                        write!(writer, "{}", sv)?;
                    },
                    Some(_) => {
                        // Unique constellation context:
                        // in V2 format, only PRN is shown
                        write!(writer, "{:2} ", sv.prn)?;
                    },
                    None => panic!("producing data with no constellation previously defined"),
                }
                write!(writer, 
                    "{} {:14.13E} {:14.13E} {:14.13E}\n", 
                    epoch.to_string_nav_v3(),
                    clk_off, 
                    clk_dr, 
                    clk_drr)?;
                // locate closest revision in db
                let db_revision = match database::closest_revision(sv.constellation, header.version) {
                    Some(v) => v,
                    _ => return Err(Error::DataBaseRevisionError),
                };
                // retrieve db items / fields to generate,
                // for this revision
                let items: Vec<_> = NAV_MESSAGES
                    .iter()
                    .filter(|r| r.constellation == sv.constellation.to_3_letter_code())
                    .map(|r| {
                        r.revisions
                            .iter()
                            .filter(|r| // identified db revision
                                u8::from_str_radix(r.major, 10).unwrap() == db_revision.major
                                && u8::from_str_radix(r.minor, 10).unwrap() == db_revision.minor
                            )
                            .map(|r| &r.items)
                            .flatten()
                    })
                    .flatten()
                    .collect();
                let mut index = 0;
                for (key, _) in items.iter() {
                    index += 1;
                    if let Some(data) = data.get(*key) {
                        write!(writer, " {}", &data.to_string())?;
                    } else { // data is missing: either not parsed or not provided
                        write!(writer, "              ")?;
                    }
                    if (index % 4) == 0 {
                        write!(writer, "\n   ")?; //TODO: do not TAB when writing last line of grouping
                    }
                }
            }
        } // EPH
        else if *class == FrameClass::SystemTimeOffset {
            for frame in frames.iter() {
                write!(writer, "> {} ", class)?;
                let (sv, sto) = frame.as_sto()
                    .unwrap();
                write!(writer, "{} LNAV\n", sv.to_string())?; //TODO LNAV or other options do exist
                write!(writer, "    {} {}    {}", &epoch.to_string_nav_v3(), sto.system, sto.utc)?;
                write!(writer, 
                    "{:14.13E} {:14.13E} {:14.13E} {:14.13E}\n",
                    sto.t_tm as f64,
                    sto.a.0,
                    sto.a.1,
                    sto.a.2)?;

            }
        } // STO
        else if *class == FrameClass::EarthOrientation {
            for frame in frames.iter() {
                let _eop = frame.as_eop()
                    .unwrap();
                panic!("NAV V4: EOP: not available yet");
                //(x, xr, xrr), (y, yr, yrr), t_tm, (dut, dutr, dutrr)) = frame.as_eop()
            }
        } // EOP
        else { // ION 
            for frame in frames.iter() {
                let (sv, msg) = frame.as_ion()
                    .unwrap();
                write!(writer, "> {} {}", class, sv)?;
                match msg {
                    IonMessage::KlobucharModel(model) => {
                        write!(writer, "IFNV\n")?;
                    },
                    IonMessage::NequickGModel(sv, model) => {
                        write!(writer, "IFNV\n")?;
                    },
                    IonMessage::BdgimModel(sv, model) => {
                        write!(writer, "IFNV\n")?;
                    },
                }
            }
        } // ION
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_is_new_epoch() {
        // NAV V<3
        let line = " 1 20 12 31 23 45  0.0 7.282570004460D-05 0.000000000000D+00 7.380000000000D+04";
        assert_eq!(is_new_epoch(line, Version::new(1, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(2, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(3, 0)), false);
        assert_eq!(is_new_epoch(line, Version::new(4, 0)), false);
        // NAV V<3
        let line = " 2 21  1  1 11 45  0.0 4.610531032090D-04 1.818989403550D-12 4.245000000000D+04";
        assert_eq!(is_new_epoch(line, Version::new(1, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(2, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(3, 0)), false);
        assert_eq!(is_new_epoch(line, Version::new(4, 0)), false);
        // GPS NAV V<3
        let line = " 3 17  1 13 23 59 44.0-1.057861372828D-04-9.094947017729D-13 0.000000000000D+00";
        assert_eq!(is_new_epoch(line, Version::new(1, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(2, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(3, 0)), false);
        assert_eq!(is_new_epoch(line, Version::new(4, 0)), false);
        // NAV V3
        let line = "C05 2021 01 01 00 00 00-4.263372393325e-04-7.525180478751e-11 0.000000000000e+00";
        assert_eq!(is_new_epoch(line, Version::new(1, 0)), false);
        assert_eq!(is_new_epoch(line, Version::new(2, 0)), false);
        assert_eq!(is_new_epoch(line, Version::new(3, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(4, 0)), false);
        // NAV V3
        let line = "R21 2022 01 01 09 15 00-2.666609361768E-04-2.728484105319E-12 5.508000000000E+05";
        assert_eq!(is_new_epoch(line, Version::new(1, 0)), false);
        assert_eq!(is_new_epoch(line, Version::new(2, 0)), false);
        assert_eq!(is_new_epoch(line, Version::new(3, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(4, 0)), false);
        // NAV V4
        let line = "> EPH G02 LNAV";
        assert_eq!(is_new_epoch(line, Version::new(2, 0)), false);
        assert_eq!(is_new_epoch(line, Version::new(3, 0)), false);
        assert_eq!(is_new_epoch(line, Version::new(4, 0)), true);
    }
    #[test]
    fn test_v2_glonass_entry() {
        let content =
" 1 20 12 31 23 45  0.0 7.282570004460D-05 0.000000000000D+00 7.380000000000D+04
   -1.488799804690D+03-2.196182250980D+00 3.725290298460D-09 0.000000000000D+00
    1.292880712890D+04-2.049269676210D+00 0.000000000000D+00 1.000000000000D+00
    2.193169775390D+04 1.059645652770D+00-9.313225746150D-10 0.000000000000D+00";
        let version = Version::new(2, 0);
        let entry = parse_epoch(version, Constellation::Glonass, content);
        assert_eq!(entry.is_ok(), true);
        let (epoch, class, frame) = entry.unwrap();
        assert_eq!(epoch, Epoch {
            date: epoch::str2date("20 12 31 23 45  0.0").unwrap(),
            flag: epoch::EpochFlag::Ok,
        });
        assert_eq!(class, FrameClass::Ephemeris);
        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);
        let (msg_type, sv, clk, clk_dr, clk_drr, map) = fr.unwrap();
        assert_eq!(msg_type, MsgType::LNAV);
        assert_eq!(sv, Sv {
            constellation: Constellation::Glonass,
            prn: 1,
        });
        assert_eq!(clk, 7.282570004460E-05);
        assert_eq!(clk_dr, 0.0); 
        assert_eq!(clk_drr, 7.38E4);
        assert_eq!(map.len(), 12);
        for (k, v) in map.iter() {
            if k.eq("satPosX") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -1.488799804690E+03);
            } else if k.eq("velX") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -2.196182250980E+00);
            } else if k.eq("accelX") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 3.725290298460E-09);
            } else if k.eq("health") {
                let v = v.as_glo_health();
                assert_eq!(v.is_some(), true);
            } else if k.eq("satPosY") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 1.292880712890E+04);
            } else if k.eq("velY") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -2.049269676210E+00);
            } else if k.eq("accelY") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.0);
            } else if k.eq("channel") {
                let v = v.as_i8();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 1);
            } else if k.eq("satPosZ") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 2.193169775390E+04);
            } else if k.eq("velZ") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 1.059645652770E+00);
            } else if k.eq("accelZ") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -9.313225746150E-10);
            } else if k.eq("ageOp") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.0);
            } else { 
                panic!("Got unexpected key \"{}\" for GLOV2 record", k);
            }
        }
    }
    #[test]
    fn test_v3_beidou_entry() {
        let content =
"C05 2021 01 01 00 00 00 -.426337239332e-03 -.752518047875e-10  .000000000000e+00
      .100000000000e+01  .118906250000e+02  .105325815814e-08 -.255139531119e+01
      .169500708580e-06  .401772442274e-03  .292365439236e-04  .649346986580e+04
      .432000000000e+06  .105705112219e-06 -.277512444499e+01 -.211410224438e-06
      .607169709798e-01 -.897671875000e+03  .154887266488e+00 -.871464871438e-10
     -.940753471872e-09  .000000000000e+00  .782000000000e+03  .000000000000e+00
      .200000000000e+01  .000000000000e+00 -.599999994133e-09 -.900000000000e-08
      .432000000000e+06  .000000000000e+00 0.000000000000e+00 0.000000000000e+00";
        let version = Version::new(3, 0);
        let entry = parse_epoch(version, Constellation::Mixed, content);
        assert_eq!(entry.is_ok(), true);
        let (epoch, class, frame) = entry.unwrap();
        assert_eq!(epoch, Epoch {
            date: epoch::str2date("2021 01 01 00 00 00").unwrap(),
            flag: epoch::EpochFlag::Ok,
        });
        assert_eq!(class, FrameClass::Ephemeris);
        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);
        let (msg_type, sv, clk, clk_dr, clk_drr, map) = fr.unwrap();
        assert_eq!(msg_type, MsgType::LNAV);
        assert_eq!(sv, Sv {
            constellation: Constellation::BeiDou,
            prn: 5,
        });
        assert_eq!(clk, -0.426337239332E-03); 
        assert_eq!(clk_dr, -0.752518047875e-10); 
        assert_eq!(clk_drr, 0.0);
        assert_eq!(map.len(), 24);
        for (k, v) in map.iter() {
            if k.eq("aode") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.100000000000e+01);
            } else if k.eq("crs") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.118906250000e+02);
            } else if k.eq("deltaN") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.105325815814e-08);
            } else if k.eq("m0") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.255139531119e+01);
            
            } else if k.eq("cuc") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.169500708580e-06);
            } else if k.eq("e") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.401772442274e-03);
            } else if k.eq("cus") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.292365439236e-04); 
            } else if k.eq("sqrta") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.649346986580e+04);
            
            } else if k.eq("toe") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.432000000000e+06); 
            } else if k.eq("cic") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.105705112219e-06);
            } else if k.eq("omega0") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.277512444499e+01);
            } else if k.eq("cis") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.211410224438e-06);
            
            } else if k.eq("i0") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.607169709798e-01);
            } else if k.eq("crc") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.897671875000e+03); 
            } else if k.eq("omega") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.154887266488e+00);
            } else if k.eq("omegaDot") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.871464871438e-10);
            
            } else if k.eq("idot") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.940753471872e-09);
            // SPARE
            } else if k.eq("bdtWeek") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.782000000000e+03); 
            //SPARE
            
            } else if k.eq("svAccuracy") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.200000000000e+01); 
            } else if k.eq("satH1") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.000000000000e+00);
            } else if k.eq("tgd1b1b3") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.599999994133e-09);
            } else if k.eq("tgd2b2b3") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.900000000000e-08);
            
            } else if k.eq("t_tm") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.432000000000e+06); 
            } else if k.eq("oadc") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.000000000000e+00); 
            } else { 
                panic!("Got unexpected key \"{}\" for BDSV3 record", k);
            }
        }
    }
    #[test]
    fn test_v3_galileo_entry() {
        let content =
"E01 2021 01 01 10 10 00 -.101553811692e-02 -.804334376880e-11  .000000000000e+00
      .130000000000e+02  .435937500000e+02  .261510892978e-08 -.142304064404e+00
      .201165676117e-05  .226471573114e-03  .109840184450e-04  .544061822701e+04
      .468600000000e+06  .111758708954e-07 -.313008275208e+01  .409781932831e-07
      .980287270202e+00  .113593750000e+03 -.276495796017e+00 -.518200156545e-08
     -.595381942905e-09  .258000000000e+03  .213800000000e+04 0.000000000000e+00
      .312000000000e+01  .000000000000e+00  .232830643654e-09  .000000000000e+00
      .469330000000e+06 0.000000000000e+00 0.000000000000e+00 0.000000000000e+00";
        let version = Version::new(3, 0);
        let entry = parse_epoch(version, Constellation::Mixed, content);
        assert_eq!(entry.is_ok(), true);
        let (epoch, class, frame) = entry.unwrap();
        assert_eq!(epoch, Epoch {
            date: epoch::str2date("2021 01 01 10 10 00").unwrap(),
            flag: epoch::EpochFlag::Ok,
        });
        assert_eq!(class, FrameClass::Ephemeris);
        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);
        let (msg_type, sv, clk, clk_dr, clk_drr, map) = fr.unwrap();
        assert_eq!(msg_type, MsgType::LNAV);
        assert_eq!(sv, Sv {
            constellation: Constellation::Galileo,
            prn: 1,
        });
        assert_eq!(clk, -0.101553811692e-02); 
        assert_eq!(clk_dr, -0.804334376880e-11);
        assert_eq!(clk_drr, 0.0);
        assert_eq!(map.len(), 24);
        for (k, v) in map.iter() {
            if k.eq("iodnav") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.130000000000e+02); 
            } else if k.eq("crs") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.435937500000e+02);
            } else if k.eq("deltaN") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.261510892978e-08);
            } else if k.eq("m0") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.142304064404e+00); 
            
            } else if k.eq("cuc") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.201165676117e-05);
            } else if k.eq("e") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.226471573114e-03); 
            } else if k.eq("cus") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.109840184450e-04); 
            } else if k.eq("sqrta") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.544061822701e+04); 
            
            } else if k.eq("toe") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.468600000000e+06); 
            } else if k.eq("cic") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.111758708954e-07); 
            } else if k.eq("omega0") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.313008275208e+01);
            } else if k.eq("cis") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.409781932831e-07);
            
            } else if k.eq("i0") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.980287270202e+00); 
            } else if k.eq("crc") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.113593750000e+03);
            } else if k.eq("omega") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.276495796017e+00);
            } else if k.eq("omegaDot") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.518200156545e-08);
            
            } else if k.eq("idot") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.595381942905e-09);
            } else if k.eq("dataSrc") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.258000000000e+03);
            } else if k.eq("galWeek") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.213800000000e+04);
            //SPARE
            
            } else if k.eq("sisa") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.312000000000e+01);
            } else if k.eq("health") {
                let v = v.as_gal_health();
                assert_eq!(v.is_some(), true);
            } else if k.eq("bgdE5aE1") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.232830643654e-09);
            } else if k.eq("bgdE5bE1") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.000000000000e+00);
            
            } else if k.eq("t_tm") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.469330000000e+06);
            
            } else { 
                panic!("Got unexpected key \"{}\" for GALV3 record", k);
            }
        }
    }
    #[test]
    fn test_v3_glonass_entry() {
        let content =
"R07 2021 01 01 09 45 00 -.420100986958e-04  .000000000000e+00  .342000000000e+05
      .124900639648e+05  .912527084351e+00  .000000000000e+00  .000000000000e+00
      .595546582031e+04  .278496932983e+01  .000000000000e+00  .500000000000e+01
      .214479208984e+05 -.131077289581e+01 -.279396772385e-08  .000000000000e+00";
        let version = Version::new(3, 0);
        let entry = parse_epoch(version, Constellation::Mixed, content);
        assert_eq!(entry.is_ok(), true);
        let (epoch, class, frame) = entry.unwrap();
        assert_eq!(epoch, Epoch {
            date: epoch::str2date("2021 01 01 09 45 00").unwrap(),
            flag: epoch::EpochFlag::Ok,
        });
        assert_eq!(class, FrameClass::Ephemeris);
        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);
        let (msg_type, sv, clk, clk_dr, clk_drr, map) = fr.unwrap();
        assert_eq!(msg_type, MsgType::LNAV);
        assert_eq!(sv, Sv {
            constellation: Constellation::Glonass,
            prn: 7,
        });
        assert_eq!(clk, -0.420100986958e-04);
        assert_eq!(clk_dr, 0.000000000000e+00);
        assert_eq!(clk_drr, 0.342000000000e+05);
        assert_eq!(map.len(), 12);
        for (k, v) in map.iter() {
            if k.eq("satPosX") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.124900639648e+05);
            } else if k.eq("velX") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.912527084351e+00);
            } else if k.eq("accelX") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.000000000000e+00);
            } else if k.eq("health") {
                let v = v.as_glo_health();
                assert_eq!(v.is_some(), true);
            } else if k.eq("satPosY") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.595546582031e+04);
            } else if k.eq("velY") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.278496932983e+01); 
            } else if k.eq("accelY") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.000000000000e+00);
            } else if k.eq("channel") {
                let v = v.as_i8();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 5);
            } else if k.eq("satPosZ") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.214479208984e+05);
            } else if k.eq("velZ") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.131077289581e+01);
            } else if k.eq("accelZ") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, -0.279396772385e-08);
            } else if k.eq("ageOp") {
                let v = v.as_f64();
                assert_eq!(v.is_some(), true);
                let v = v.unwrap();
                assert_eq!(v, 0.000000000000e+00);
            } else { 
                panic!("Got unexpected key \"{}\" for GLOV3 record", k);
            }
        }
    }
/* GAL V4 from example please */
}
