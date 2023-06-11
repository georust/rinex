//! `NavigationData` parser and related methods
use crate::processing::{
    Filter, Interpolate, Mask, MaskFilter, MaskOperand, Preprocessing, TargetItem,
};
use regex::{Captures, Regex};
use std::collections::BTreeMap;
use std::str::FromStr;
use thiserror::Error;

/*
 * When formatting floating point number in Navigation RINEX,
 * exponent are expected to be in the %02d form,
 * but Rust is only capable of formating %d (AFAIK).
 * With this macro, we simply rework all exponents encountered in a string
 */
fn double_exponent_digits(content: &str) -> String {
    // replace "eN " with "E+0N"
    let re = Regex::new(r"e\d{1} ").unwrap();
    let lines = re.replace_all(&content, |caps: &Captures| format!("E+0{}", &caps[0][1..]));

    // replace "eN" with "E+0N"
    let re = Regex::new(r"e\d{1}").unwrap();
    let lines = re.replace_all(&lines, |caps: &Captures| format!("E+0{}", &caps[0][1..]));

    // replace "e-N " with "E-0N"
    let re = Regex::new(r"e-\d{1} ").unwrap();
    let lines = re.replace_all(&lines, |caps: &Captures| format!("E-0{}", &caps[0][2..]));

    // replace "e-N" with "e-0N"
    let re = Regex::new(r"e-\d{1}").unwrap();
    let lines = re.replace_all(&lines, |caps: &Captures| format!("E-0{}", &caps[0][2..]));

    lines.to_string()
}

use crate::{
    epoch, gnss_time::GnssTime, merge, merge::Merge, prelude::*, split, split::Split, sv,
    types::Type, version::Version,
};

use super::{
    eopmessage, ephemeris, ionmessage,
    orbits::{closest_revision, OrbitItemError, NAV_ORBITS},
    stomessage, BdModel, EopMessage, Ephemeris, IonMessage, KbModel, NgModel, StoMessage,
};

use hifitime::Duration;

/// Possible Navigation Frame declinations for an epoch
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FrameClass {
    Ephemeris,
    SystemTimeOffset,
    EarthOrientation,
    IonosphericModel,
}

impl Default for FrameClass {
    fn default() -> Self {
        Self::Ephemeris
    }
}

impl std::fmt::Display for FrameClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Ephemeris => f.write_str("EPH"),
            Self::SystemTimeOffset => f.write_str("STO"),
            Self::EarthOrientation => f.write_str("EOP"),
            Self::IonosphericModel => f.write_str("ION"),
        }
    }
}

impl std::str::FromStr for FrameClass {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = s.to_uppercase();
        let c = c.trim();
        match c {
            "EPH" => Ok(Self::Ephemeris),
            "STO" => Ok(Self::SystemTimeOffset),
            "EOP" => Ok(Self::EarthOrientation),
            "ION" => Ok(Self::IonosphericModel),
            _ => Err(Error::UnknownFrameClass),
        }
    }
}

/// Navigation Message Types
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MsgType {
    /// Legacy NAV
    LNAV,
    /// FDMA
    FDMA,
    /// FNAV
    FNAV,
    /// INAV
    INAV,
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
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::LNAV => f.write_str("LNAV"),
            Self::FNAV => f.write_str("FNAV"),
            Self::INAV => f.write_str("INAV"),
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

impl std::str::FromStr for MsgType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = s.to_uppercase();
        let c = c.trim();
        match c {
            "LNAV" => Ok(Self::LNAV),
            "FDMA" => Ok(Self::FDMA),
            "FNAV" => Ok(Self::FNAV),
            "INAV" => Ok(Self::INAV),
            "IFNV" => Ok(Self::IFNV),
            "D1" => Ok(Self::D1),
            "D2" => Ok(Self::D2),
            "D1D2" => Ok(Self::D1D2),
            "SBAS" => Ok(Self::SBAS),
            "CNVX" => Ok(Self::CNVX),
            _ => Err(Error::UnknownMsgType),
        }
    }
}

/// Navigation Frame for a given epoch
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Frame {
    /// Ephemeris for a given Vehicle `Sv`
    Eph(MsgType, Sv, Ephemeris),
    /// Earth Orientation Parameters
    Eop(MsgType, Sv, EopMessage),
    /// Ionospheric Model Message
    Ion(MsgType, Sv, IonMessage),
    /// System Time Offset Message
    Sto(MsgType, Sv, StoMessage),
}

impl Default for Frame {
    fn default() -> Self {
        Self::Eph(MsgType::default(), Sv::default(), Ephemeris::default())
    }
}

impl Frame {
    /// Unwraps self as Ephemeris frame
    pub fn as_eph(&self) -> Option<(&MsgType, &Sv, &Ephemeris)> {
        match self {
            Self::Eph(msg, sv, eph) => Some((msg, sv, eph)),
            _ => None,
        }
    }
    /// Unwraps self as mutable Ephemeris frame reference
    pub fn as_mut_eph(&mut self) -> Option<(&mut MsgType, &mut Sv, &mut Ephemeris)> {
        match self {
            Self::Eph(msg, sv, eph) => Some((msg, sv, eph)),
            _ => None,
        }
    }
    /// Unwraps self as Ionospheric Model frame
    pub fn as_ion(&self) -> Option<(&MsgType, &Sv, &IonMessage)> {
        match self {
            Self::Ion(msg, sv, fr) => Some((msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self as mutable Ionospheric Model frame reference
    pub fn as_mut_ion(&mut self) -> Option<(&mut MsgType, &mut Sv, &mut IonMessage)> {
        match self {
            Self::Ion(msg, sv, fr) => Some((msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self as Earth Orientation frame
    pub fn as_eop(&self) -> Option<(&MsgType, &Sv, &EopMessage)> {
        match self {
            Self::Eop(msg, sv, fr) => Some((msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self as Mutable Earth Orientation frame reference
    pub fn as_mut_eop(&mut self) -> Option<(&mut MsgType, &mut Sv, &mut EopMessage)> {
        match self {
            Self::Eop(msg, sv, fr) => Some((msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self as System Time Offset frame
    pub fn as_sto(&self) -> Option<(&MsgType, &Sv, &StoMessage)> {
        match self {
            Self::Sto(msg, sv, fr) => Some((msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self as mutable System Time Offset frame reference
    pub fn as_mut_sto(&mut self) -> Option<(&MsgType, &mut Sv, &mut StoMessage)> {
        match self {
            Self::Sto(msg, sv, fr) => Some((msg, sv, fr)),
            _ => None,
        }
    }
}

/// Navigation Record.
/// Data is sorted by epoch, and by Frame class.
/// ```
/// use rinex::prelude::*;
/// use rinex::navigation::*;
/// let rnx = Rinex::from_file("../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx")
///     .unwrap();
/// let record = rnx.record.as_nav()
///     .unwrap();
/// for (epoch, classes) in record {
///     for (class, frames) in classes {
///         if *class == FrameClass::Ephemeris {
///             // Ephemeris are the most common Navigation Frames
///             // Up until V3 they were the only existing frame type.
///             // Refer to [ephemeris::Ephemeris] for an example of use
///         }
///         else if *class == FrameClass::IonosphericModel {
///             // Modern Navigation frame, see [ionmessage::IonMessage]
///         }
///         else if *class == FrameClass::SystemTimeOffset {
///             // Modern Navigation frame, see [stomessage::StoMessage]
///         }
///         else if *class == FrameClass::EarthOrientation {
///             // Modern Navigation frame, see [eopmessage::EopMessage]
///         }
///     }
/// }
/// ```
pub type Record = BTreeMap<Epoch, BTreeMap<FrameClass, Vec<Frame>>>;

/// Returns true if given content matches the beginning of a
/// Navigation record epoch
pub(crate) fn is_new_epoch(line: &str, v: Version) -> bool {
    if v.major < 3 {
        // old RINEX
        if line.len() < 23 {
            return false; // not enough bytes
                          // to describe a PRN and an Epoch
        }
        let (prn, _) = line.split_at(2);
        // 1st entry is a valid integer number
        if u8::from_str_radix(prn.trim(), 10).is_err() {
            return false;
        }
        // rest matches a valid epoch descriptor
        let datestr = &line[3..22];
        epoch::parse(&datestr).is_ok()
    } else if v.major == 3 {
        // RINEX V3
        if line.len() < 24 {
            return false; // not enough bytes
                          // to describe an SVN and an Epoch
        }
        // 1st entry matches a valid SV description
        let (sv, _) = line.split_at(4);
        if Sv::from_str(sv).is_err() {
            return false;
        }
        // rest matches a valid epoch descriptor
        let datestr = &line[4..23];
        epoch::parse(&datestr).is_ok()
    } else {
        // Modern --> easy
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
    OrbitRevision,
    #[error("unknown nav frame class")]
    UnknownFrameClass,
    #[error("unknown nav message type")]
    UnknownMsgType,
    #[error("failed to parse msg type")]
    SvError(#[from] sv::Error),
    #[error("failed to parse orbit field")]
    ParseOrbitError(#[from] OrbitItemError),
    #[error("failed to parse sv::prn")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse sv clock fields")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse epoch")]
    EpochError(#[from] epoch::Error),
    #[error("failed to identify class/type")]
    StrumError(#[from] strum::ParseError),
    #[error("failed to parse EPH message")]
    EphMessageError(#[from] ephemeris::Error),
    #[error("failed to parse ION message")]
    IonMessageError(#[from] ionmessage::Error),
    #[error("failed to parse EOP message")]
    EopMessageError(#[from] eopmessage::Error),
    #[error("failed to parse STO message")]
    StoMessageError(#[from] stomessage::Error),
}

/// Builds `Record` entry for `NavigationData`
pub(crate) fn parse_epoch(
    version: Version,
    constell: Constellation,
    content: &str,
) -> Result<(Epoch, FrameClass, Frame), Error> {
    if content.starts_with(">") {
        parse_v4_record_entry(content)
    } else {
        parse_v2_v3_record_entry(version, constell, content)
    }
}

/// Builds `Record` entry for Modern NAV frames
fn parse_v4_record_entry(content: &str) -> Result<(Epoch, FrameClass, Frame), Error> {
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
            let (epoch, _, ephemeris) = Ephemeris::parse_v4(lines)?;
            (epoch, Frame::Eph(msg_type, sv, ephemeris))
        },
        FrameClass::SystemTimeOffset => {
            let (epoch, msg) = StoMessage::parse(lines)?;
            (epoch, Frame::Sto(msg_type, sv, msg))
        },
        FrameClass::EarthOrientation => {
            let (epoch, msg) = EopMessage::parse(lines)?;
            (epoch, Frame::Eop(msg_type, sv, msg))
        },
        FrameClass::IonosphericModel => {
            let (epoch, msg): (Epoch, IonMessage) = match msg_type {
                MsgType::IFNV => {
                    let (epoch, model) = NgModel::parse(lines)?;
                    (epoch, IonMessage::NequickGModel(model))
                },
                MsgType::CNVX => match sv.constellation {
                    Constellation::BeiDou => {
                        let (epoch, model) = BdModel::parse(lines)?;
                        (epoch, IonMessage::BdgimModel(model))
                    },
                    _ => {
                        let (epoch, model) = KbModel::parse(lines)?;
                        (epoch, IonMessage::KlobucharModel(model))
                    },
                },
                _ => {
                    let (epoch, model) = KbModel::parse(lines)?;
                    (epoch, IonMessage::KlobucharModel(model))
                },
            };
            (epoch, Frame::Ion(msg_type, sv, msg))
        },
    };
    Ok((epoch, frame_class, fr))
}

fn parse_v2_v3_record_entry(
    version: Version,
    constell: Constellation,
    content: &str,
) -> Result<(Epoch, FrameClass, Frame), Error> {
    let (epoch, sv, ephemeris) = Ephemeris::parse_v2v3(version, constell, content.lines())?;
    let fr = Frame::Eph(MsgType::LNAV, sv, ephemeris);
    Ok((epoch, FrameClass::Ephemeris, fr))
}

/*
 * Reworks generated/formatted line to match standards
 */
fn fmt_rework(major: u8, lines: &str) -> String {
    /*
     * There's an issue when formatting the exponent 00 in XXXXX.E00
     * Rust does not know how to format an exponent on multiples digits,
     * and RINEX expects two.
     * If we try to rework this line, it may corrupt some SVNN fields.
     */
    let mut lines = double_exponent_digits(lines);

    if major < 3 {
        /*
         * In old RINEX, D+00 D-01 is used instead of E+00 E-01
         */
        lines = lines.replace("E-", "D-");
        lines = lines.replace("E+", "D+");
    }
    lines.to_string()
}

/// Writes given epoch into stream
pub(crate) fn fmt_epoch(
    epoch: &Epoch,
    data: &BTreeMap<FrameClass, Vec<Frame>>,
    header: &Header,
) -> Result<String, Error> {
    if header.version.major < 4 {
        fmt_epoch_v2v3(epoch, data, header)
    } else {
        fmt_epoch_v4(epoch, data, header)
    }
}

fn fmt_epoch_v2v3(
    epoch: &Epoch,
    data: &BTreeMap<FrameClass, Vec<Frame>>,
    header: &Header,
) -> Result<String, Error> {
    let mut lines = String::with_capacity(128);
    for (class, frames) in data.iter() {
        if *class == FrameClass::Ephemeris {
            for frame in frames.iter() {
                let (_, sv, ephemeris) = frame.as_eph().unwrap();
                match &header.constellation {
                    Some(Constellation::Mixed) => {
                        // Mixed constellation context
                        // we need to fully describe the vehicle
                        lines.push_str(&format!("{} ", sv));
                    },
                    Some(_) => {
                        // Unique constellation context:
                        // in V2 format, only PRN is shown
                        lines.push_str(&format!("{:2} ", sv.prn));
                    },
                    None => {
                        panic!("can't generate data without predefined constellations");
                    },
                }
                lines.push_str(&format!(
                    "{} ",
                    epoch::format(*epoch, None, Type::NavigationData, header.version.major)
                ));
                lines.push_str(&format!(
                    "{:14.11E} {:14.11E} {:14.11E}\n   ",
                    ephemeris.clock_bias, ephemeris.clock_drift, ephemeris.clock_drift_rate
                ));
                if header.version.major == 3 {
                    lines.push_str("  ");
                }
                // locate closest revision in db
                let orbits_revision = match closest_revision(sv.constellation, header.version) {
                    Some(v) => v,
                    _ => return Err(Error::OrbitRevision),
                };
                // retrieve db items / fields to generate,
                // for this revision
                let orbits_standards: Vec<_> = NAV_ORBITS
                    .iter()
                    .filter(|r| r.constellation == sv.constellation.to_3_letter_code())
                    .map(|r| {
                        r.revisions
                            .iter()
                            .filter(|r| // identified db revision
                                u8::from_str_radix(r.major, 10).unwrap() == orbits_revision.major
                                && u8::from_str_radix(r.minor, 10).unwrap() == orbits_revision.minor
                            )
                            .map(|r| &r.items)
                            .flatten()
                    })
                    .flatten()
                    .collect();
                let nb_items_per_line = 4;
                let mut chunks = orbits_standards.chunks_exact(nb_items_per_line).peekable();
                while let Some(chunk) = chunks.next() {
                    if chunks.peek().is_some() {
                        for (key, _) in chunk {
                            if let Some(data) = ephemeris.orbits.get(*key) {
                                lines.push_str(&format!("{} ", data.to_string()));
                            } else {
                                lines.push_str(&format!("                   "));
                            }
                        }
                        lines.push_str(&format!("\n     "));
                    } else {
                        // last row
                        for (key, _) in chunk {
                            if let Some(data) = ephemeris.orbits.get(*key) {
                                lines.push_str(&format!("{}", data.to_string()));
                            } else {
                                lines.push_str(&format!("                   "));
                            }
                        }
                        lines.push_str("\n");
                    }
                }
            }
        }
    }
    lines = fmt_rework(header.version.major, &lines);
    Ok(lines)
}

fn fmt_epoch_v4(
    epoch: &Epoch,
    data: &BTreeMap<FrameClass, Vec<Frame>>,
    header: &Header,
) -> Result<String, Error> {
    let mut lines = String::with_capacity(128);
    for (class, frames) in data.iter() {
        if *class == FrameClass::Ephemeris {
            for frame in frames.iter() {
                let (msgtype, sv, ephemeris) = frame.as_eph().unwrap();
                lines.push_str(&format!("> {} {} {}\n", class, sv, msgtype));
                match &header.constellation {
                    Some(Constellation::Mixed) => {
                        // Mixed constellation context
                        // we need to fully describe the vehicle
                        lines.push_str(&sv.to_string());
                        lines.push_str(" ");
                    },
                    Some(_) => {
                        // Unique constellation context:
                        // in V2 format, only PRN is shown
                        lines.push_str(&format!("{:02} ", sv.prn));
                    },
                    None => panic!("producing data with no constellation previously defined"),
                }
                lines.push_str(&format!(
                    "{} ",
                    epoch::format(*epoch, None, Type::NavigationData, header.version.major)
                ));
                lines.push_str(&format!(
                    "{:14.13E} {:14.13E} {:14.13E}\n",
                    ephemeris.clock_bias, ephemeris.clock_drift, ephemeris.clock_drift_rate
                ));
                // locate closest revision in db
                let orbits_revision = match closest_revision(sv.constellation, header.version) {
                    Some(v) => v,
                    _ => return Err(Error::OrbitRevision),
                };
                // retrieve db items / fields to generate,
                // for this revision
                let orbits_standards: Vec<_> = NAV_ORBITS
                    .iter()
                    .filter(|r| r.constellation == sv.constellation.to_3_letter_code())
                    .map(|r| {
                        r.revisions
                            .iter()
                            .filter(|r| // identified db revision
                                u8::from_str_radix(r.major, 10).unwrap() == orbits_revision.major
                                && u8::from_str_radix(r.minor, 10).unwrap() == orbits_revision.minor
                            )
                            .map(|r| &r.items)
                            .flatten()
                    })
                    .flatten()
                    .collect();
                let mut index = 0;
                for (key, _) in orbits_standards.iter() {
                    index += 1;
                    if let Some(data) = ephemeris.orbits.get(*key) {
                        lines.push_str(&format!(" {}", data.to_string()));
                    } else {
                        // data is missing: either not parsed or not provided
                        lines.push_str("              ");
                    }
                    if (index % 4) == 0 {
                        lines.push_str("\n   "); //TODO: do not TAB when writing last line of grouping
                    }
                }
            }
        }
        // EPH
        else if *class == FrameClass::SystemTimeOffset {
            for frame in frames.iter() {
                let (msg, sv, sto) = frame.as_sto().unwrap();
                lines.push_str(&format!("> {} {} {}\n", class, sv, msg));
                lines.push_str(&format!(
                    "    {} {}    {}\n",
                    epoch::format(*epoch, None, Type::NavigationData, header.version.major),
                    sto.system,
                    sto.utc
                ));
                lines.push_str(&format!(
                    "   {:14.13E} {:14.13E} {:14.13E} {:14.13E}\n",
                    sto.t_tm as f64, sto.a.0, sto.a.1, sto.a.2
                ));
            }
        }
        // STO
        else if *class == FrameClass::EarthOrientation {
            for frame in frames.iter() {
                let _eop = frame.as_eop().unwrap();
                panic!("NAV V4: EOP: not available yet");
                //(x, xr, xrr), (y, yr, yrr), t_tm, (dut, dutr, dutrr)) = frame.as_eop()
            }
        }
        // EOP
        else {
            // ION
            for frame in frames.iter() {
                let (msg, sv, ion) = frame.as_ion().unwrap();
                lines.push_str(&format!("> {} {} {}\n", class, sv, msg));
                match ion {
                    IonMessage::KlobucharModel(_model) => {},
                    IonMessage::NequickGModel(_model) => {},
                    IonMessage::BdgimModel(_model) => {},
                }
            }
        } // ION
    }
    lines = fmt_rework(4, &lines);
    Ok(lines)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_is_new_epoch() {
        // NAV V<3
        let line =
            " 1 20 12 31 23 45  0.0 7.282570004460D-05 0.000000000000D+00 7.380000000000D+04";
        assert_eq!(is_new_epoch(line, Version::new(1, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(2, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(3, 0)), false);
        assert_eq!(is_new_epoch(line, Version::new(4, 0)), false);
        // NAV V<3
        let line =
            " 2 21  1  1 11 45  0.0 4.610531032090D-04 1.818989403550D-12 4.245000000000D+04";
        assert_eq!(is_new_epoch(line, Version::new(1, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(2, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(3, 0)), false);
        assert_eq!(is_new_epoch(line, Version::new(4, 0)), false);
        // GPS NAV V<3
        let line =
            " 3 17  1 13 23 59 44.0-1.057861372828D-04-9.094947017729D-13 0.000000000000D+00";
        assert_eq!(is_new_epoch(line, Version::new(1, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(2, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(3, 0)), false);
        assert_eq!(is_new_epoch(line, Version::new(4, 0)), false);
        // NAV V3
        let line =
            "C05 2021 01 01 00 00 00-4.263372393325e-04-7.525180478751e-11 0.000000000000e+00";
        assert_eq!(is_new_epoch(line, Version::new(1, 0)), false);
        assert_eq!(is_new_epoch(line, Version::new(2, 0)), false);
        assert_eq!(is_new_epoch(line, Version::new(3, 0)), true);
        assert_eq!(is_new_epoch(line, Version::new(4, 0)), false);
        // NAV V3
        let line =
            "R21 2022 01 01 09 15 00-2.666609361768E-04-2.728484105319E-12 5.508000000000E+05";
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
        assert_eq!(
            epoch,
            Epoch::from_gregorian_utc(2020, 12, 31, 23, 45, 00, 00)
        );
        assert_eq!(class, FrameClass::Ephemeris);
        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);
        let (msg_type, sv, ephemeris) = fr.unwrap();
        assert_eq!(msg_type, &MsgType::LNAV);
        assert_eq!(
            sv,
            &Sv {
                constellation: Constellation::Glonass,
                prn: 1,
            }
        );
        assert_eq!(ephemeris.clock_bias, 7.282570004460E-05);
        assert_eq!(ephemeris.clock_drift, 0.0);
        assert_eq!(ephemeris.clock_drift_rate, 7.38E4);
        let orbits = &ephemeris.orbits;
        assert_eq!(orbits.len(), 12);
        for (k, v) in orbits.iter() {
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
        assert_eq!(
            epoch,
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 00, 00, 00)
        );
        assert_eq!(class, FrameClass::Ephemeris);
        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);
        let (msg_type, sv, ephemeris) = fr.unwrap();
        assert_eq!(msg_type, &MsgType::LNAV);
        assert_eq!(
            sv,
            &Sv {
                constellation: Constellation::BeiDou,
                prn: 5,
            }
        );
        assert_eq!(ephemeris.clock_bias, -0.426337239332E-03);
        assert_eq!(ephemeris.clock_drift, -0.752518047875e-10);
        assert_eq!(ephemeris.clock_drift_rate, 0.0);
        let orbits = &ephemeris.orbits;
        assert_eq!(orbits.len(), 24);
        for (k, v) in orbits.iter() {
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
            } else if k.eq("aodc") {
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
        assert_eq!(
            epoch,
            Epoch::from_gregorian_utc(2021, 01, 01, 10, 10, 00, 00)
        );
        assert_eq!(class, FrameClass::Ephemeris);
        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);
        let (msg_type, sv, ephemeris) = fr.unwrap();
        assert_eq!(msg_type, &MsgType::LNAV);
        assert_eq!(
            sv,
            &Sv {
                constellation: Constellation::Galileo,
                prn: 1,
            }
        );
        assert_eq!(ephemeris.clock_bias, -0.101553811692e-02);
        assert_eq!(ephemeris.clock_drift, -0.804334376880e-11);
        assert_eq!(ephemeris.clock_drift_rate, 0.0);
        let orbits = &ephemeris.orbits;
        assert_eq!(orbits.len(), 24);
        for (k, v) in orbits.iter() {
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
        assert_eq!(
            epoch,
            Epoch::from_gregorian_utc(2021, 01, 01, 09, 45, 00, 00)
        );
        assert_eq!(class, FrameClass::Ephemeris);
        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);
        let (msg_type, sv, ephemeris) = fr.unwrap();
        assert_eq!(msg_type, &MsgType::LNAV);
        assert_eq!(
            sv,
            &Sv {
                constellation: Constellation::Glonass,
                prn: 7,
            }
        );
        assert_eq!(ephemeris.clock_bias, -0.420100986958e-04);
        assert_eq!(ephemeris.clock_drift, 0.000000000000e+00);
        assert_eq!(ephemeris.clock_drift_rate, 0.342000000000e+05);
        let orbits = &ephemeris.orbits;
        assert_eq!(orbits.len(), 12);
        for (k, v) in orbits.iter() {
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
    #[test]
    fn test_fmt_rework() {
        let content = "1000123  -123123e-1 -1.23123123e0 -0.123123e-4";
        assert_eq!(
            fmt_rework(2, content),
            "1000123  -123123D-01 -1.23123123D+00 -0.123123D-04"
        );
        assert_eq!(
            fmt_rework(3, content),
            "1000123  -123123E-01 -1.23123123E+00 -0.123123E-04"
        );
    }
}

impl Merge for Record {
    /// Merges `rhs` into `Self` without mutable access at the expense of more memcopies
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self`
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        for (rhs_epoch, rhs_classes) in rhs.iter() {
            if let Some(lhs_classes) = self.get_mut(rhs_epoch) {
                for (rhs_class, rhs_frames) in rhs_classes.iter() {
                    if let Some(lhs_frames) = lhs_classes.get_mut(rhs_class) {
                        for frame in rhs_frames {
                            if !lhs_frames.contains(frame) {
                                // complete new frame
                                lhs_frames.push(frame.clone());
                            }
                        }
                    } else {
                        // new frame class
                        lhs_classes.insert(*rhs_class, rhs_frames.clone());
                    }
                }
            } else {
                // new epoch
                self.insert(*rhs_epoch, rhs_classes.clone());
            }
        }
        Ok(())
    }
}

impl Split for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let r0 = self
            .iter()
            .flat_map(|(k, v)| {
                if k < &epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self
            .iter()
            .flat_map(|(k, v)| {
                if k >= &epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        Ok((r0, r1))
    }
    fn split_dt(&self, _duration: Duration) -> Result<Vec<Self>, split::Error> {
        Ok(Vec::new())
    }
}

impl GnssTime for Record {
    fn timeseries(&self, dt: Duration) -> TimeSeries {
        let epochs: Vec<_> = self.keys().collect();
        TimeSeries::inclusive(
            **epochs.get(0).expect("failed to determine first epoch"),
            **epochs
                .get(epochs.len() - 1)
                .expect("failed to determine last epoch"),
            dt,
        )
    }
    fn convert_timescale(&mut self, ts: TimeScale) {
        self.iter_mut()
            .map(|(k, v)| (k.in_time_scale(ts), v))
            .count();
    }
    fn with_timescale(&self, ts: TimeScale) -> Self {
        let mut s = self.clone();
        s.convert_timescale(ts);
        s
    }
}

impl Mask for Record {
    fn mask(&self, mask: MaskFilter) -> Self {
        let mut s = self.clone();
        s.mask_mut(mask);
        s
    }
    fn mask_mut(&mut self, mask: MaskFilter) {
        match mask.operand {
            MaskOperand::Equals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e == epoch),
                TargetItem::SvItem(filter) => {
                    self.retain(|_, classes| {
                        classes.retain(|class, frames| {
                            if *class == FrameClass::Ephemeris {
                                frames.retain(|fr| {
                                    let (_, sv, _) = fr.as_eph().unwrap();
                                    filter.contains(&sv)
                                });
                                frames.len() > 0
                            } else {
                                true // do not affect other frame types
                            }
                        });
                        classes.len() > 0
                    });
                },
                TargetItem::ConstellationItem(filter) => {
                    self.retain(|_, classes| {
                        classes.retain(|class, frames| {
                            if *class == FrameClass::Ephemeris {
                                frames.retain(|fr| {
                                    let (_, sv, _) = fr.as_eph().unwrap();
                                    filter.contains(&sv.constellation)
                                });
                                frames.len() > 0
                            } else {
                                true // do not affect other frame types
                            }
                        });
                        classes.len() > 0
                    });
                },
                TargetItem::OrbitItem(filter) => {
                    self.retain(|_, classes| {
                        classes.retain(|class, frames| {
                            if *class == FrameClass::Ephemeris {
                                frames.retain_mut(|fr| {
                                    let (_, _, ephemeris) = fr.as_mut_eph().unwrap();
                                    let orbits = &mut ephemeris.orbits;
                                    orbits.retain(|k, _| filter.contains(&k));
                                    orbits.len() > 0
                                });
                                frames.len() > 0
                            } else {
                                true // do not affect other frame types
                            }
                        });
                        classes.len() > 0
                    });
                },
                TargetItem::NavFrameItem(filter) => {
                    self.retain(|_, classes| {
                        classes.retain(|class, _| filter.contains(&class));
                        classes.len() > 0
                    });
                },
                TargetItem::NavMsgItem(filter) => {
                    self.retain(|_, classes| {
                        classes.retain(|class, frames| {
                            if *class == FrameClass::Ephemeris {
                                frames.retain(|fr| {
                                    let (msg, _, _) = fr.as_eph().unwrap();
                                    filter.contains(&msg)
                                });
                            } else if *class == FrameClass::SystemTimeOffset {
                                frames.retain(|fr| {
                                    let (msg, _, _) = fr.as_sto().unwrap();
                                    filter.contains(&msg)
                                });
                            } else if *class == FrameClass::IonosphericModel {
                                frames.retain(|fr| {
                                    let (msg, _, _) = fr.as_ion().unwrap();
                                    filter.contains(&msg)
                                });
                            } else {
                                frames.retain(|fr| {
                                    let (msg, _, _) = fr.as_eop().unwrap();
                                    filter.contains(&msg)
                                });
                            }
                            frames.len() > 0
                        });
                        classes.len() > 0
                    });
                },
                _ => {}, // TargetItem::
            },
            MaskOperand::NotEquals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e != epoch),
                TargetItem::ConstellationItem(filter) => {
                    self.retain(|_, classes| {
                        classes.retain(|class, frames| {
                            if *class == FrameClass::Ephemeris {
                                frames.retain(|fr| {
                                    let (_, sv, _) = fr.as_eph().unwrap();
                                    !filter.contains(&sv.constellation)
                                });
                                frames.len() > 0
                            } else {
                                true // do not affect other frame types
                            }
                        });
                        classes.len() > 0
                    });
                },
                TargetItem::SvItem(filter) => {
                    self.retain(|_, classes| {
                        classes.retain(|class, frames| {
                            if *class == FrameClass::Ephemeris {
                                frames.retain(|fr| {
                                    let (_, sv, _) = fr.as_eph().unwrap();
                                    !filter.contains(&sv)
                                });
                                frames.len() > 0
                            } else {
                                true // do not affect other frame types
                            }
                        });
                        classes.len() > 0
                    });
                },
                TargetItem::OrbitItem(filter) => {
                    self.retain(|_, classes| {
                        classes.retain(|class, frames| {
                            if *class == FrameClass::Ephemeris {
                                frames.retain_mut(|fr| {
                                    let (_, _, ephemeris) = fr.as_mut_eph().unwrap();
                                    let orbits = &mut ephemeris.orbits;
                                    orbits.retain(|k, _| !filter.contains(&k));
                                    orbits.len() > 0
                                });
                                frames.len() > 0
                            } else {
                                true // do not affect other frame types
                            }
                        });
                        classes.len() > 0
                    });
                },
                _ => {}, // TargetItem::
            },
            MaskOperand::GreaterThan => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e > epoch),
                TargetItem::SvItem(filter) => {
                    self.retain(|_, classes| {
                        classes.retain(|class, frames| {
                            if *class == FrameClass::Ephemeris {
                                frames.retain(|fr| {
                                    let mut retain = false;
                                    let (_, sv, _) = fr.as_eph().unwrap();
                                    for item in &filter {
                                        if item.constellation == sv.constellation {
                                            retain = sv.prn > item.prn;
                                        } else {
                                            retain = true;
                                        }
                                    }
                                    retain
                                });
                                frames.len() > 0
                            } else {
                                true // do not affect other frame types
                            }
                        });
                        classes.len() > 0
                    });
                },
                _ => {}, // TargetItem::
            },
            MaskOperand::GreaterEquals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e >= epoch),
                TargetItem::SvItem(filter) => {
                    self.retain(|_, classes| {
                        classes.retain(|class, frames| {
                            if *class == FrameClass::Ephemeris {
                                frames.retain(|fr| {
                                    let mut retain = false;
                                    let (_, sv, _) = fr.as_eph().unwrap();
                                    for item in &filter {
                                        if item.constellation == sv.constellation {
                                            retain = sv.prn >= item.prn;
                                        } else {
                                            retain = true;
                                        }
                                    }
                                    retain
                                });
                                frames.len() > 0
                            } else {
                                true // do not affect other frame types
                            }
                        });
                        classes.len() > 0
                    });
                },
                _ => {}, // TargetItem::
            },
            MaskOperand::LowerThan => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e < epoch),
                TargetItem::SvItem(filter) => {
                    self.retain(|_, classes| {
                        classes.retain(|class, frames| {
                            if *class == FrameClass::Ephemeris {
                                frames.retain(|fr| {
                                    let mut retain = false;
                                    let (_, sv, _) = fr.as_eph().unwrap();
                                    for item in &filter {
                                        if item.constellation == sv.constellation {
                                            retain = sv.prn < item.prn;
                                        } else {
                                            retain = true;
                                        }
                                    }
                                    retain
                                });
                                frames.len() > 0
                            } else {
                                true // do not affect other frame types
                            }
                        });
                        classes.len() > 0
                    });
                },
                _ => {}, // TargetItem::
            },
            MaskOperand::LowerEquals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e <= epoch),
                TargetItem::SvItem(filter) => {
                    self.retain(|_, classes| {
                        classes.retain(|class, frames| {
                            if *class == FrameClass::Ephemeris {
                                frames.retain(|fr| {
                                    let mut retain = false;
                                    let (_, sv, _) = fr.as_eph().unwrap();
                                    for item in &filter {
                                        if item.constellation == sv.constellation {
                                            retain = sv.prn > item.prn;
                                        } else {
                                            retain = true;
                                        }
                                    }
                                    retain
                                });
                                frames.len() > 0
                            } else {
                                true // do not affect other frame types
                            }
                        });
                        classes.len() > 0
                    });
                },
                _ => {}, // TargetItem::
            },
        }
    }
}

/*
 * Decimates only a given record subset
 */
fn decimate_data_subset(record: &mut Record, subset: &Record, target: &TargetItem) {
    match target {
        TargetItem::SvItem(svs) => {
            /*
             * remove Sv data that should now be missing
             */
            for (epoch, classes) in record.iter_mut() {
                if subset.get(epoch).is_none() {
                    // should be missing
                    for (class, frames) in classes.iter_mut() {
                        if *class == FrameClass::Ephemeris {
                            // does not apply to other frames @ the moment
                            frames.retain(|fr| {
                                let (_msg_type, sv, _ephemeris) = fr.as_eph().unwrap();
                                svs.contains(sv)
                            });
                        }
                    }
                }
            }
        },
        TargetItem::ConstellationItem(constells_list) => {
            /*
             * Removes ephemeris frames that should now be missing
             */
            for (epoch, classes) in record.iter_mut() {
                if subset.get(epoch).is_none() {
                    // should be missing
                    for (class, frames) in classes.iter_mut() {
                        if *class == FrameClass::Ephemeris {
                            // does not apply to other frames @ the moment
                            frames.retain(|fr| {
                                let (_msg_type, sv, _ephemeris) = fr.as_eph().unwrap();
                                constells_list.contains(&sv.constellation)
                            });
                        }
                    }
                }
            }
        },
        TargetItem::OrbitItem(orbit_fields) => {
            /*
             * Removes ephemeris frames that should now be missing
             */
            for (epoch, classes) in record.iter_mut() {
                if subset.get(epoch).is_none() {
                    // should be missing
                    for (class, frames) in classes.iter_mut() {
                        if *class == FrameClass::Ephemeris {
                            // does not apply to other frames @ the moment
                            frames.retain_mut(|fr| {
                                let (_msg_type, _sv, ephemeris) = fr.as_mut_eph().unwrap();
                                ephemeris.orbits.retain(|k, _| orbit_fields.contains(&k));
                                ephemeris.orbits.len() > 0
                            });
                        }
                    }
                }
            }
        },
        TargetItem::AzimuthItem(_azim) => {
            unimplemented!("navigation:record:decimate_data_subset(azim)");
        },
        TargetItem::ElevationItem(_elev) => {
            unimplemented!("navigation:record:decimate_data_subset(elev)");
        },
        TargetItem::NavFrameItem(_frame_classes) => {
            unimplemented!("navigation:record:decimate_data_subset(navframe)");
        },
        TargetItem::NavMsgItem(_msg_types) => {
            unimplemented!("navigation:record:decimate_data_subset(navmsg)");
        },
        _ => {}, // does not apply
    }
}

impl Preprocessing for Record {
    fn filter(&self, f: Filter) -> Self {
        let mut s = self.clone();
        s.filter_mut(f);
        s
    }
    fn filter_mut(&mut self, filt: Filter) {
        match filt {
            Filter::Mask(mask) => self.mask_mut(mask),
            Filter::Interp(filter) => self.interpolate_mut(filter.series),
            Filter::Decimation(filter) => match filter.dtype {
                DecimationType::DecimByRatio(r) => {
                    if filter.target.is_none() {
                        self.decimate_by_ratio_mut(r);
                        return; // no need to proceed further
                    }

                    let item = filter.target.unwrap();

                    // apply mask to retain desired subset
                    let mask = MaskFilter {
                        item: item.clone(),
                        operand: MaskOperand::Equals,
                    };

                    // decimate
                    let subset = self.mask(mask).decimate_by_ratio(r);
                    // adapt self's subset to new data rate
                    decimate_data_subset(self, &subset, &item);
                },
                DecimationType::DecimByInterval(dt) => {
                    if filter.target.is_none() {
                        self.decimate_by_interval_mut(dt);
                        return; // no need to proceed further
                    }

                    let item = filter.target.unwrap();

                    // apply mask to retain desired subset
                    let mask = MaskFilter {
                        item: item.clone(),
                        operand: MaskOperand::Equals,
                    };

                    // decimate
                    let subset = self.mask(mask).decimate_by_interval(dt);
                    // adapt self's subset to new data rate
                    decimate_data_subset(self, &subset, &item);
                },
            },
            Filter::Smoothing(_) => unimplemented!("navigation:record:smoothing"),
        }
    }
}

impl Interpolate for Record {
    fn interpolate(&self, series: TimeSeries) -> Self {
        let mut s = self.clone();
        s.interpolate_mut(series);
        s
    }
    fn interpolate_mut(&mut self, _series: TimeSeries) {
        unimplemented!("navigation:record:interpolate_mut()")
    }
}

use crate::processing::{Decimate, DecimationType};

impl Decimate for Record {
    /// Decimates Self by desired factor
    fn decimate_by_ratio_mut(&mut self, r: u32) {
        let mut i = 0;
        self.retain(|_, _| {
            let retained = (i % r) == 0;
            i += 1;
            retained
        });
    }
    /// Copies and Decimates Self by desired factor
    fn decimate_by_ratio(&self, r: u32) -> Self {
        let mut s = self.clone();
        s.decimate_by_ratio_mut(r);
        s
    }
    /// Decimates Self to fit minimum epoch interval
    fn decimate_by_interval_mut(&mut self, interval: Duration) {
        let mut last_retained: Option<Epoch> = None;
        self.retain(|e, _| {
            if last_retained.is_some() {
                let dt = *e - last_retained.unwrap();
                last_retained = Some(*e);
                dt > interval
            } else {
                last_retained = Some(*e);
                true // always retain 1st epoch
            }
        });
    }
    fn decimate_by_interval(&self, dt: Duration) -> Self {
        let mut s = self.clone();
        s.decimate_by_interval_mut(dt);
        s
    }
    fn decimate_match_mut(&mut self, rhs: &Self) {
        self.retain(|e, _| rhs.get(e).is_some());
    }
    fn decimate_match(&self, rhs: &Self) -> Self {
        let mut s = self.clone();
        s.decimate_match_mut(&rhs);
        s
    }
}
