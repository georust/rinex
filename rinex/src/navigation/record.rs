//! `NavigationData` parser and related methods
use thiserror::Error;
use std::str::FromStr;
use strum_macros::EnumString;
use regex::{Regex, Captures};
use std::collections::BTreeMap;

use crate::{
	Epoch, 
	epoch, 
    epoch::str2date,
	sv,
	Header,
	Constellation, Sv,
	version::Version,
    merge, merge::Merge,
    split, split::Split,
};

use super::{
	ephemeris, Ephemeris,
    eopmessage, EopMessage, 
    stomessage, StoMessage, 
	ionmessage, IonMessage, 
	BdModel, KbModel, NgModel,
	orbits::{
        NAV_ORBITS,
        closest_revision,
        OrbitItemError,
    },
};

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
    fn default() -> Self {
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
    /// Ephemeris for a given Vehicule `Sv`
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
    pub fn as_eph (&self) -> Option<(&MsgType, &Sv, &Ephemeris)> {
        match self {
            Self::Eph(msg, sv, eph) => Some((msg, sv, eph)),
            _ => None,
        }
    }
    /// Unwraps self as mutable Ephemeris frame reference
    pub fn as_mut_eph (&mut self) -> Option<(&mut MsgType, &mut Sv, &mut Ephemeris)> {
        match self {
            Self::Eph(msg, sv, eph) => Some((msg, sv, eph)),
            _ => None,
        }
    }
    /// Unwraps self as Ionospheric Model frame
    pub fn as_ion (&self) -> Option<(&MsgType, &Sv, &IonMessage)> {
        match self {
            Self::Ion(msg, sv, fr) => Some((msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self as mutable Ionospheric Model frame reference
    pub fn as_mut_ion (&mut self) -> Option<(&mut MsgType, &mut Sv, &mut IonMessage)> {
        match self {
            Self::Ion(msg, sv, fr) => Some((msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self as Earth Orientation frame
    pub fn as_eop (&self) -> Option<(&MsgType, &Sv, &EopMessage)> {
        match self {
            Self::Eop(msg, sv, fr) => Some((msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self as Mutable Earth Orientation frame reference
    pub fn as_mut_eop (&mut self) -> Option<(&mut MsgType, &mut Sv, &mut EopMessage)> {
        match self {
            Self::Eop(msg, sv, fr) => Some((msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self as System Time Offset frame
    pub fn as_sto (&self) -> Option<(&MsgType, &Sv, &StoMessage)> {
        match self {
            Self::Sto(msg, sv, fr) => Some((msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self as mutable System Time Offset frame reference
    pub fn as_mut_sto (&mut self) -> Option<(&MsgType, &mut Sv, &mut StoMessage)> {
        match self {
            Self::Sto(msg, sv, fr) => Some((msg, sv, fr)),
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
        str2date(&datestr).is_ok()

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
        str2date(&datestr).is_ok()

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
    OrbitRevision,
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
pub fn parse_epoch (version: Version, constell: Constellation, content: &str) ->
        Result<(Epoch, FrameClass, Frame), Error>
{
    if content.starts_with(">") {
        parse_v4_record_entry(content)
    } else {
        parse_v2_v3_record_entry(version, constell, content)
    }
}

/// Builds `Record` entry for Modern NAV frames
fn parse_v4_record_entry (content: &str) ->
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
                MsgType::CNVX => {
                    match sv.constellation {
                        Constellation::BeiDou => {
                            let (epoch, model) = BdModel::parse(lines)?;
                            (epoch, IonMessage::BdgimModel(model))
                        },
                        _ => {
                            let (epoch, model) = KbModel::parse(lines)?;
                            (epoch, IonMessage::KlobucharModel(model))
                        },
                    }
                },
                _ => {
                    let (epoch, model) = KbModel::parse(lines)?;
                    (epoch, IonMessage::KlobucharModel(model))
                }
            };
            (epoch, Frame::Ion(msg_type, sv, msg))
        },
    };
    Ok((epoch, frame_class, fr))
}

fn parse_v2_v3_record_entry (version: Version, constell: Constellation, content: &str) ->
        Result<(Epoch, FrameClass, Frame), Error>
{
	let (epoch, sv, ephemeris) = Ephemeris::parse_v2v3(version, constell, content.lines())?;
    let fr = Frame::Eph(
		MsgType::LNAV, 
		sv, 
		ephemeris);
    Ok((
		epoch,
        FrameClass::Ephemeris,
        fr,
    ))
}

/// Reworks / formats to match NAV RINEX standards
/// mainly replaces E+/- exponents by D+/-
/// also exponent must use two digits
fn fmt_rework_v2(lines: &str) -> String {
    let lines = lines.replace("E-", "D-");
    let lines = lines.replace("E+", "D+");
    let lines = lines.replace("E", "D+");
    //lazy_static! {
    // static ref
        let re = Regex::new(r"[D][\+]\d{1}").unwrap();
        let lines = re.replace(&lines, |caps: &Captures| {
            format!("D+0{}", &caps[0][2..])
        });
        
        let re = Regex::new(r"[D][\-]\d{1}").unwrap();
        let lines = re.replace(&lines, |caps: &Captures| {
            format!("D-0{}", &caps[0][2..])
        });
    //}
    lines.to_string()
}

/// Writes given epoch into stream 
pub fn fmt_epoch (
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

fn fmt_epoch_v2v3 (
        epoch: &Epoch, 
        data: &BTreeMap<FrameClass, Vec<Frame>>,
        header: &Header,
    ) -> Result<String, Error> {
    let mut lines = String::with_capacity(128);
    for (class, frames) in data.iter() {
        if *class == FrameClass::Ephemeris {
            for frame in frames.iter() {
                let (_, sv, ephemeris) = frame.as_eph()
                    .unwrap();
                match &header.constellation {
                    Some(Constellation::Mixed) => {
                        // Mixed constellation context
                        // we need to fully describe the vehicule
                        lines.push_str(&format!("{} ", sv));
                    },
                    Some(_) => {
                        // Unique constellation context:
                        // in V2 format, only PRN is shown
                        lines.push_str(&format!("{:2} ", sv.prn));
                    },
                    None => {
                        panic!("producing data with no constellation previously defined");
                    },
                }
                if header.version.major < 3 {
                    lines.push_str(&format!("{:e} ", epoch));
                } else {
                    lines.push_str(&format!("{:E} ", epoch));
                }
                lines.push_str(&format!(
                    "{:14.12E} {:14.12E} {:14.12E}\n   ",
                    ephemeris.clock_bias,
                    ephemeris.clock_drift,
                    ephemeris.clock_drift_rate));
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
                let mut chunks = orbits_standards 
                    .chunks_exact(nb_items_per_line)
                    .peekable();
                while let Some(chunk) = chunks.next() {
                    if chunks.peek().is_some() {
                        for (key, _) in chunk {
                            if let Some(data) = ephemeris.orbits.get(*key) {
                                lines.push_str(&format!("{} ", data.to_string()));
                            } else {
                                lines.push_str(&format!("              "));
                            }
                        }
                        lines.push_str(&format!("\n     "));
                    } else { // last row
                        for (key, _) in chunk {
                            if let Some(data) = ephemeris.orbits.get(*key) {
                                lines.push_str(&format!("{}", data.to_string()));
                            } else {
                                lines.push_str(&format!("              "));
                            }
                        }
                        lines.push_str("\n");
                    }
                }
            }
        }
    }
    if header.version.major < 3 {
        lines = fmt_rework_v2(&lines);
    }
    Ok(lines)
}

fn fmt_epoch_v4 (
        epoch: &Epoch, 
        data: &BTreeMap<FrameClass, Vec<Frame>>,
        header: &Header,
    ) -> Result<String, Error> {
    let mut lines = String::with_capacity(128);
    for (class, frames) in data.iter() {
        if *class == FrameClass::Ephemeris {
            for frame in frames.iter() {
                let (msgtype, sv, ephemeris) = frame.as_eph()
                    .unwrap();
                lines.push_str(&format!("> {} {} {}\n", class, sv, msgtype));
                match &header.constellation {
                    Some(Constellation::Mixed) => {
                        // Mixed constellation context
                        // we need to fully describe the vehicule
                        lines.push_str(&sv.to_string());
                    },
                    Some(_) => {
                        // Unique constellation context:
                        // in V2 format, only PRN is shown
                        lines.push_str(&format!("{:2} ", sv.prn));
                    },
                    None => panic!("producing data with no constellation previously defined"),
                }
                lines.push_str(&format!(
                    "{:E} {:14.13E} {:14.13E} {:14.13E}\n", 
                    epoch,
                    ephemeris.clock_bias, 
                    ephemeris.clock_drift, 
                    ephemeris.clock_drift_rate));
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
                    } else { // data is missing: either not parsed or not provided
                        lines.push_str("              ");
                    }
                    if (index % 4) == 0 {
                        lines.push_str("\n   "); //TODO: do not TAB when writing last line of grouping
                    }
                }
            }
        } // EPH
        else if *class == FrameClass::SystemTimeOffset {
            for frame in frames.iter() {
                let (msg, sv, sto) = frame.as_sto()
                    .unwrap();
                lines.push_str(&format!("> {} {} {}\n", class, sv, msg));
                lines.push_str(&format!("    {:e} {}    {}", epoch, sto.system, sto.utc));
                lines.push_str(&format!( 
                    "{:14.13E} {:14.13E} {:14.13E} {:14.13E}\n",
                    sto.t_tm as f64,
                    sto.a.0,
                    sto.a.1,
                    sto.a.2));

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
                let (msg, sv, ion) = frame.as_ion()
                    .unwrap();
                lines.push_str(&format!("> {} {} {}\n", class, sv, msg));
                match ion {
                    IonMessage::KlobucharModel(_model) => {

                    },
                    IonMessage::NequickGModel(_model) => {

                    },
                    IonMessage::BdgimModel(_model) => {

                    },
                }
            }
        } // ION
    }
    Ok(lines)
}

#[cfg(test)]
mod test {
    use super::*;
	use crate::EpochFlag;
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
            epoch: str2date("20 12 31 23 45  0.0").unwrap(),
            flag: EpochFlag::Ok,
        });
        assert_eq!(class, FrameClass::Ephemeris);
        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);
        let (msg_type, sv, ephemeris) = fr.unwrap();
        assert_eq!(msg_type, &MsgType::LNAV);
        assert_eq!(sv, &Sv {
            constellation: Constellation::Glonass,
            prn: 1,
        });
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
        assert_eq!(epoch, Epoch {
            epoch: str2date("2021 01 01 00 00 00").unwrap(),
            flag: EpochFlag::Ok,
        });
        assert_eq!(class, FrameClass::Ephemeris);
        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);
        let (msg_type, sv, ephemeris) = fr.unwrap();
        assert_eq!(msg_type, &MsgType::LNAV);
        assert_eq!(sv, &Sv {
            constellation: Constellation::BeiDou,
            prn: 5,
        });
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
            epoch: str2date("2021 01 01 10 10 00").unwrap(),
            flag: EpochFlag::Ok,
        });
        assert_eq!(class, FrameClass::Ephemeris);
        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);
        let (msg_type, sv, ephemeris) = fr.unwrap();
        assert_eq!(msg_type, &MsgType::LNAV);
        assert_eq!(sv, &Sv {
            constellation: Constellation::Galileo,
            prn: 1,
        });
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
        assert_eq!(epoch, Epoch {
            epoch: str2date("2021 01 01 09 45 00").unwrap(),
            flag: EpochFlag::Ok,
        });
        assert_eq!(class, FrameClass::Ephemeris);
        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);
        let (msg_type, sv, ephemeris) = fr.unwrap();
        assert_eq!(msg_type, &MsgType::LNAV);
        assert_eq!(sv, &Sv {
            constellation: Constellation::Glonass,
            prn: 7,
        });
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
/* GAL V4 from example please */
}

impl Merge<Record> for Record {
    /// Merges `rhs` into `Self` without mutable access at the expense of more memcopies
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self`
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        for (epoch, classes) in rhs.iter() {
            if let Some(cclasses) = self.get_mut(epoch) {
                for (class, frames) in classes.iter() {
                    if let Some(fframes) = cclasses.get_mut(class) {
                        for frame in frames {
                            // add missing frames
                            if !fframes.contains(frame) {
                                fframes.push(frame.clone());
                            }
                        }
                    } else { // new frame class
                        cclasses.insert(*class, frames.clone());
                    }
                }
            } else { // new epoch
                self.insert(*epoch, classes.clone());
            }
        }
        Ok(())
    }
}

impl Split<Record> for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let r0 = self.iter()
            .flat_map(|(k, v)| {
                if k < &epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self.iter()
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
}
