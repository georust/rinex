//! NAV frames parser
use super::{Error, FrameClass};
use regex::{Captures, Regex};
use std::collections::BTreeMap;
use std::str::FromStr;

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
    epoch, gnss_time::GnssTime, merge, merge::Merge, prelude::*, split, split::Split, types::Type,
    version::Version,
};

use super::{
    orbits::{closest_revision, NAV_ORBITS},
    BdModel, EopMessage, Ephemeris, IonMessage, KbModel, NgModel, StoMessage,
};

use hifitime::Duration;

/// Navigation Message Types
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum NavMsgType {
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

impl Default for NavMsgType {
    fn default() -> Self {
        Self::LNAV
    }
}

impl std::fmt::Display for NavMsgType {
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

impl std::str::FromStr for NavMsgType {
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
            _ => Err(Error::UnknownNavMsgType),
        }
    }
}

/// Navigation Frame published at a certain Epoch
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum NavFrame {
    /// Ephemeris for given [`Sv`]
    Eph(NavMsgType, Sv, Ephemeris),
    /// Earth Orientation Parameters
    Eop(NavMsgType, Sv, EopMessage),
    /// Ionospheric Model
    Ion(NavMsgType, Sv, IonMessage),
    /// System Time Offset
    Sto(NavMsgType, Sv, StoMessage),
}

impl Default for NavFrame {
    fn default() -> Self {
        Self::Eph(NavMsgType::default(), Sv::default(), Ephemeris::default())
    }
}

impl NavFrame {
    /// Unwraps self, if possible, as ([`NavMsgType`], [`Sv`], [`Ephemeris`])
    pub fn as_eph(&self) -> Option<(NavMsgType, &Sv, &Ephemeris)> {
        match self {
            Self::Eph(msg, sv, eph) => Some((*msg, sv, eph)),
            _ => None,
        }
    }
    /// Unwraps self, if possible, as mutable ([`NavMsgType`], [`Sv`], [`Ephemeris`])
    pub fn as_mut_eph(&mut self) -> Option<(NavMsgType, &mut Sv, &mut Ephemeris)> {
        match self {
            Self::Eph(msg, sv, eph) => Some((*msg, sv, eph)),
            _ => None,
        }
    }
    /// Unwraps self, if possible, as ([`NavMsgType`], [`Sv`], [`IonMessage`])
    pub fn as_ion(&self) -> Option<(NavMsgType, &Sv, &IonMessage)> {
        match self {
            Self::Ion(msg, sv, fr) => Some((*msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self, if possible, as mutable ([`NavMsgType`], [`Sv`], [`IonMessage`])
    pub fn as_mut_ion(&mut self) -> Option<(NavMsgType, &mut Sv, &mut IonMessage)> {
        match self {
            Self::Ion(msg, sv, fr) => Some((*msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self, if possible, as ([`NavMsgType`], [`Sv`], [`EopMessage`])
    pub fn as_eop(&self) -> Option<(NavMsgType, &Sv, &EopMessage)> {
        match self {
            Self::Eop(msg, sv, fr) => Some((*msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self, if possible, as mutable ([`NavMsgType`], [`Sv`], [`EopMessage`])
    pub fn as_mut_eop(&mut self) -> Option<(NavMsgType, &mut Sv, &mut EopMessage)> {
        match self {
            Self::Eop(msg, sv, fr) => Some((*msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self, if possible, as ([`NavMsgType`], [`Sv`], [`StoMessage`])
    pub fn as_sto(&self) -> Option<(NavMsgType, &Sv, &StoMessage)> {
        match self {
            Self::Sto(msg, sv, fr) => Some((*msg, sv, fr)),
            _ => None,
        }
    }
    /// Unwraps self, if possible, as mutable ([`NavMsgType`], [`Sv`], [`StoMessage`])
    pub fn as_mut_sto(&mut self) -> Option<(NavMsgType, &mut Sv, &mut StoMessage)> {
        match self {
            Self::Sto(msg, sv, fr) => Some((*msg, sv, fr)),
            _ => None,
        }
    }
}

/// Navigation Record content:
/// data is sorted by [`Epoch`] and wrapped in a [`NavFrame`]
pub type Record = BTreeMap<Epoch, Vec<NavFrame>>;

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

/// Builds `Record` entry for `NavigationData`
pub(crate) fn parse_epoch(
    version: Version,
    constell: Constellation,
    content: &str,
) -> Result<(Epoch, NavFrame), Error> {
    if content.starts_with(">") {
        parse_v4_record_entry(content)
    } else {
        parse_v2_v3_record_entry(version, constell, content)
    }
}

/// Builds `Record` entry for Modern NAV frames
fn parse_v4_record_entry(content: &str) -> Result<(Epoch, NavFrame), Error> {
    let mut lines = content.lines();
    let line = match lines.next() {
        Some(l) => l,
        _ => return Err(Error::MissingData),
    };

    let (_, rem) = line.split_at(2);
    let (frame_class, rem) = rem.split_at(4);
    let (svnn, rem) = rem.split_at(4);

    // parse marker: defines which frame type will follow
    let frame_class = FrameClass::from_str(frame_class.trim())?;
    let sv = Sv::from_str(svnn.trim())?;
    let msg_type = NavMsgType::from_str(rem.trim())?;

    let (epoch, fr): (Epoch, NavFrame) = match frame_class {
        FrameClass::Ephemeris => {
            let (epoch, _, ephemeris) = Ephemeris::parse_v4(lines)?;
            (epoch, NavFrame::Eph(msg_type, sv, ephemeris))
        },
        FrameClass::SystemTimeOffset => {
            let (epoch, msg) = StoMessage::parse(lines)?;
            (epoch, NavFrame::Sto(msg_type, sv, msg))
        },
        FrameClass::EarthOrientation => {
            let (epoch, msg) = EopMessage::parse(lines)?;
            (epoch, NavFrame::Eop(msg_type, sv, msg))
        },
        FrameClass::IonosphericModel => {
            let (epoch, msg): (Epoch, IonMessage) = match msg_type {
                NavMsgType::IFNV => {
                    let (epoch, model) = NgModel::parse(lines)?;
                    (epoch, IonMessage::NequickGModel(model))
                },
                NavMsgType::CNVX => match sv.constellation {
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
            (epoch, NavFrame::Ion(msg_type, sv, msg))
        },
    };
    Ok((epoch, fr))
}

fn parse_v2_v3_record_entry(
    version: Version,
    constell: Constellation,
    content: &str,
) -> Result<(Epoch, NavFrame), Error> {
    // NAV V2/V3 only contain Ephemeris frames
    let (epoch, sv, ephemeris) = Ephemeris::parse_v2v3(version, constell, content.lines())?;
    // Wrap Ephemeris into a NavFrame
    let fr = NavFrame::Eph(NavMsgType::LNAV, sv, ephemeris);
    Ok((epoch, fr))
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
    data: &Vec<NavFrame>,
    header: &Header,
) -> Result<String, Error> {
    if header.version.major < 4 {
        fmt_epoch_v2v3(epoch, data, header)
    } else {
        fmt_epoch_v4(epoch, data, header)
    }
}

fn fmt_epoch_v2v3(epoch: &Epoch, data: &Vec<NavFrame>, header: &Header) -> Result<String, Error> {
    let mut lines = String::with_capacity(128);
    for fr in data.iter() {
        if let Some(fr) = fr.as_eph() {
            let (_, sv, ephemeris) = fr;
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
    lines = fmt_rework(header.version.major, &lines);
    Ok(lines)
}

fn fmt_epoch_v4(epoch: &Epoch, data: &Vec<NavFrame>, header: &Header) -> Result<String, Error> {
    let mut lines = String::with_capacity(128);
    for fr in data.iter() {
        if let Some(fr) = fr.as_eph() {
            let (msgtype, sv, ephemeris) = fr;
            lines.push_str(&format!("> {} {} {}\n", FrameClass::Ephemeris, sv, msgtype));
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
                            && u8::from_str_radix(r.minor, 10).unwrap() == orbits_revision.minor)
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
        } else if let Some(fr) = fr.as_sto() {
            let (msg, sv, sto) = fr;
            lines.push_str(&format!(
                "> {} {} {}\n",
                FrameClass::SystemTimeOffset,
                sv,
                msg
            ));
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
        } else if let Some(_fr) = fr.as_eop() {
            todo!("NAV V4: EOP: we have no example as of today");
            //(x, xr, xrr), (y, yr, yrr), t_tm, (dut, dutr, dutrr)) = frame.as_eop()
        }
        // EOP
        else if let Some(fr) = fr.as_ion() {
            let (msg, sv, ion) = fr;
            lines.push_str(&format!(
                "> {} {} {}\n",
                FrameClass::EarthOrientation,
                sv,
                msg
            ));
            match ion {
                IonMessage::KlobucharModel(_model) => todo!("ION:Kb"),
                IonMessage::NequickGModel(_model) => todo!("ION:Ng"),
                IonMessage::BdgimModel(_model) => todo!("ION:Bd"),
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
    fn new_epoch() {
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
    fn parse_glonass_v2() {
        let content =
            " 1 20 12 31 23 45  0.0 7.282570004460D-05 0.000000000000D+00 7.380000000000D+04
   -1.488799804690D+03-2.196182250980D+00 3.725290298460D-09 0.000000000000D+00
    1.292880712890D+04-2.049269676210D+00 0.000000000000D+00 1.000000000000D+00
    2.193169775390D+04 1.059645652770D+00-9.313225746150D-10 0.000000000000D+00";
        let version = Version::new(2, 0);
        let entry = parse_epoch(version, Constellation::Glonass, content);
        assert_eq!(entry.is_ok(), true);

        let (epoch, frame) = entry.unwrap();
        assert_eq!(
            epoch,
            Epoch::from_gregorian_utc(2020, 12, 31, 23, 45, 00, 00)
        );

        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);

        let (msg_type, sv, ephemeris) = fr.unwrap();
        assert_eq!(msg_type, NavMsgType::LNAV);
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
    fn parse_beidou_v3() {
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

        let (epoch, frame) = entry.unwrap();
        assert_eq!(
            epoch,
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 00, 00, 00)
        );

        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);

        let (msg_type, sv, ephemeris) = fr.unwrap();
        assert_eq!(msg_type, NavMsgType::LNAV);
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
    fn parse_galileo_v3() {
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

        let (epoch, frame) = entry.unwrap();
        assert_eq!(
            epoch,
            Epoch::from_gregorian_utc(2021, 01, 01, 10, 10, 00, 00)
        );

        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);

        let (msg_type, sv, ephemeris) = fr.unwrap();
        assert_eq!(msg_type, NavMsgType::LNAV);
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
    fn parse_glonass_v3() {
        let content =
            "R07 2021 01 01 09 45 00 -.420100986958e-04  .000000000000e+00  .342000000000e+05
      .124900639648e+05  .912527084351e+00  .000000000000e+00  .000000000000e+00
      .595546582031e+04  .278496932983e+01  .000000000000e+00  .500000000000e+01
      .214479208984e+05 -.131077289581e+01 -.279396772385e-08  .000000000000e+00";
        let version = Version::new(3, 0);
        let entry = parse_epoch(version, Constellation::Mixed, content);
        assert_eq!(entry.is_ok(), true);
        let (epoch, frame) = entry.unwrap();
        assert_eq!(
            epoch,
            Epoch::from_gregorian_utc(2021, 01, 01, 09, 45, 00, 00)
        );

        let fr = frame.as_eph();
        assert_eq!(fr.is_some(), true);
        let (msg_type, sv, ephemeris) = fr.unwrap();
        assert_eq!(msg_type, NavMsgType::LNAV);
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
    fn format_rework() {
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
        for (rhs_epoch, rhs_frames) in rhs {
            if let Some(frames) = self.get_mut(&rhs_epoch) {
                // this epoch already exists
                for fr in rhs_frames {
                    if !frames.contains(&fr) {
                        frames.push(fr.clone()); // insert new NavFrame
                    }
                }
            } else {
                // insert new epoch
                self.insert(*rhs_epoch, rhs_frames.clone());
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

#[cfg(feature = "processing")]
use crate::preprocessing::*;

fn mask_mut_equal(rec: &mut Record, target: TargetItem) {
    match target {
        TargetItem::EpochItem(epoch) => rec.retain(|e, _| *e == epoch),
        TargetItem::SvItem(filter) => {
            rec.retain(|_, frames| {
                frames.retain(|fr| {
                    if let Some((_, sv, _)) = fr.as_eph() {
                        filter.contains(&sv)
                    } else if let Some((_, sv, _)) = fr.as_ion() {
                        filter.contains(&sv)
                    } else if let Some((_, sv, _)) = fr.as_eop() {
                        filter.contains(&sv)
                    } else if let Some((_, sv, _)) = fr.as_sto() {
                        filter.contains(&sv)
                    } else {
                        // non existing
                        false
                    }
                });
                frames.len() > 0
            });
        },
        TargetItem::ConstellationItem(filter) => {
            rec.retain(|_, frames| {
                frames.retain(|fr| {
                    if let Some((_, sv, _)) = fr.as_eph() {
                        filter.contains(&sv.constellation)
                    } else if let Some((_, sv, _)) = fr.as_ion() {
                        filter.contains(&sv.constellation)
                    } else if let Some((_, sv, _)) = fr.as_eop() {
                        filter.contains(&sv.constellation)
                    } else if let Some((_, sv, _)) = fr.as_sto() {
                        filter.contains(&sv.constellation)
                    } else {
                        // non existing
                        false
                    }
                });
                frames.len() > 0
            });
        },
        TargetItem::OrbitItem(_filter) => {
            unimplemented!("orbititem filter");
            //self.retain(|_, frames| {
            //    frames.retain(|fr| {
            //        if let Some((_, _, ephemeris)) = fr.as_mut_eph() {
            //            let orbits = &mut ephemeris.orbits;
            //            orbits.retain(|k, _| filter.contains(&k));
            //            orbits.len() > 0
            //        } else { // other frames do not have "orbits"
            //            false
            //        }
            //    });
            //    frames.len() > 0
            //});
        },
        TargetItem::NavFrameItem(filter) => {
            rec.retain(|_, frames| {
                frames.retain(|fr| {
                    if let Some(_) = fr.as_eph() {
                        filter.contains(&FrameClass::Ephemeris)
                    } else if let Some(_) = fr.as_eop() {
                        filter.contains(&FrameClass::EarthOrientation)
                    } else if let Some(_) = fr.as_ion() {
                        filter.contains(&FrameClass::IonosphericModel)
                    } else if let Some(_) = fr.as_sto() {
                        filter.contains(&FrameClass::SystemTimeOffset)
                    } else {
                        false
                    }
                });
                frames.len() > 0
            });
        },
        TargetItem::NavMsgItem(filter) => {
            rec.retain(|_, frames| {
                frames.retain(|fr| {
                    if let Some((msg, _, _)) = fr.as_eph() {
                        filter.contains(&msg)
                    } else if let Some((msg, _, _)) = fr.as_ion() {
                        filter.contains(&msg)
                    } else if let Some((msg, _, _)) = fr.as_eop() {
                        filter.contains(&msg)
                    } else if let Some((msg, _, _)) = fr.as_sto() {
                        filter.contains(&msg)
                    } else {
                        false
                    }
                });
                frames.len() > 0
            });
        },
        _ => {}, // Other items: either not supported, or do not apply
    }
}

fn mask_mut_ineq(rec: &mut Record, target: TargetItem) {
    match target {
        TargetItem::EpochItem(epoch) => rec.retain(|e, _| *e != epoch),
        TargetItem::SvItem(filter) => {
            rec.retain(|_, frames| {
                frames.retain(|fr| {
                    if let Some((_, sv, _)) = fr.as_eph() {
                        !filter.contains(&sv)
                    } else if let Some((_, sv, _)) = fr.as_ion() {
                        !filter.contains(&sv)
                    } else if let Some((_, sv, _)) = fr.as_eop() {
                        !filter.contains(&sv)
                    } else if let Some((_, sv, _)) = fr.as_sto() {
                        !filter.contains(&sv)
                    } else {
                        // non existing
                        false
                    }
                });
                frames.len() > 0
            });
        },
        TargetItem::ConstellationItem(filter) => {
            rec.retain(|_, frames| {
                frames.retain(|fr| {
                    if let Some((_, sv, _)) = fr.as_eph() {
                        !filter.contains(&sv.constellation)
                    } else if let Some((_, sv, _)) = fr.as_ion() {
                        !filter.contains(&sv.constellation)
                    } else if let Some((_, sv, _)) = fr.as_eop() {
                        !filter.contains(&sv.constellation)
                    } else if let Some((_, sv, _)) = fr.as_sto() {
                        !filter.contains(&sv.constellation)
                    } else {
                        // non existing
                        false
                    }
                });
                frames.len() > 0
            });
        },
        TargetItem::OrbitItem(_filter) => {
            unimplemented!("orbititem filter");
            //rec.retain(|_, frames| {
            //    frames.retain(|fr| {
            //        if let Some((_, _, ephemeris)) = fr.as_mut_eph() {
            //            let orbits = &mut ephemeris.orbits;
            //            orbits.retain(|k, _| filter.contains(&k));
            //            orbits.len() > 0
            //        } else { // other frames do not have "orbits"
            //            false
            //        }
            //    });
            //    frames.len() > 0
            //});
        },
        TargetItem::NavFrameItem(filter) => {
            rec.retain(|_, frames| {
                frames.retain(|fr| {
                    if let Some(_) = fr.as_eph() {
                        !filter.contains(&FrameClass::Ephemeris)
                    } else if let Some(_) = fr.as_eop() {
                        !filter.contains(&FrameClass::EarthOrientation)
                    } else if let Some(_) = fr.as_ion() {
                        !filter.contains(&FrameClass::IonosphericModel)
                    } else if let Some(_) = fr.as_sto() {
                        !filter.contains(&FrameClass::SystemTimeOffset)
                    } else {
                        false
                    }
                });
                frames.len() > 0
            });
        },
        TargetItem::NavMsgItem(filter) => {
            rec.retain(|_, frames| {
                frames.retain(|fr| {
                    if let Some((msg, _, _)) = fr.as_eph() {
                        !filter.contains(&msg)
                    } else if let Some((msg, _, _)) = fr.as_ion() {
                        !filter.contains(&msg)
                    } else if let Some((msg, _, _)) = fr.as_eop() {
                        !filter.contains(&msg)
                    } else if let Some((msg, _, _)) = fr.as_sto() {
                        !filter.contains(&msg)
                    } else {
                        false
                    }
                });
                frames.len() > 0
            });
        },
        _ => {}, // Other items: either not supported, or do not apply
    }
}

fn mask_mut_leq(rec: &mut Record, target: TargetItem) {
    match target {
        TargetItem::EpochItem(epoch) => rec.retain(|e, _| *e <= epoch),
        TargetItem::SvItem(filter) => {
            // for each constell, grab PRN#
            let filter: Vec<_> = filter.iter().map(|sv| (sv.constellation, sv.prn)).collect();

            rec.retain(|_, frames| {
                frames.retain(|fr| {
                    if let Some((_, sv, _)) = fr.as_eph() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn <= *prn;
                            }
                        }
                        pass
                    } else if let Some((_, sv, _)) = fr.as_ion() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn <= *prn;
                            }
                        }
                        pass
                    } else if let Some((_, sv, _)) = fr.as_eop() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn <= *prn;
                            }
                        }
                        pass
                    } else if let Some((_, sv, _)) = fr.as_sto() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn <= *prn;
                            }
                        }
                        pass
                    } else {
                        // non existing
                        false
                    }
                });
                frames.len() > 0
            });
        },
        _ => {}, // Other items: either not supported, or do not apply
    }
}

fn mask_mut_lt(rec: &mut Record, target: TargetItem) {
    match target {
        TargetItem::EpochItem(epoch) => rec.retain(|e, _| *e < epoch),
        TargetItem::SvItem(filter) => {
            // for each constell, grab PRN#
            let filter: Vec<_> = filter.iter().map(|sv| (sv.constellation, sv.prn)).collect();

            rec.retain(|_, frames| {
                frames.retain(|fr| {
                    if let Some((_, sv, _)) = fr.as_eph() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn < *prn;
                            }
                        }
                        pass
                    } else if let Some((_, sv, _)) = fr.as_ion() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn < *prn;
                            }
                        }
                        pass
                    } else if let Some((_, sv, _)) = fr.as_eop() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn < *prn;
                            }
                        }
                        pass
                    } else if let Some((_, sv, _)) = fr.as_sto() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn < *prn;
                            }
                        }
                        pass
                    } else {
                        // non existing
                        false
                    }
                });
                frames.len() > 0
            });
        },
        _ => {}, // Other items: either not supported, or do not apply
    }
}

fn mask_mut_gt(rec: &mut Record, target: TargetItem) {
    match target {
        TargetItem::EpochItem(epoch) => rec.retain(|e, _| *e > epoch),
        TargetItem::SvItem(filter) => {
            // for each constell, grab PRN#
            let filter: Vec<_> = filter.iter().map(|sv| (sv.constellation, sv.prn)).collect();

            rec.retain(|_, frames| {
                frames.retain(|fr| {
                    if let Some((_, sv, _)) = fr.as_eph() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn > *prn;
                            }
                        }
                        pass
                    } else if let Some((_, sv, _)) = fr.as_ion() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn > *prn;
                            }
                        }
                        pass
                    } else if let Some((_, sv, _)) = fr.as_eop() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn > *prn;
                            }
                        }
                        pass
                    } else if let Some((_, sv, _)) = fr.as_sto() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn > *prn;
                            }
                        }
                        pass
                    } else {
                        // non existing
                        false
                    }
                });
                frames.len() > 0
            });
        },
        _ => {}, // Other items: either not supported, or do not apply
    }
}

fn mask_mut_geq(rec: &mut Record, target: TargetItem) {
    match target {
        TargetItem::EpochItem(epoch) => rec.retain(|e, _| *e >= epoch),
        TargetItem::SvItem(filter) => {
            // for each constell, grab PRN#
            let filter: Vec<_> = filter.iter().map(|sv| (sv.constellation, sv.prn)).collect();

            rec.retain(|_, frames| {
                frames.retain(|fr| {
                    if let Some((_, sv, _)) = fr.as_eph() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn >= *prn;
                            }
                        }
                        pass
                    } else if let Some((_, sv, _)) = fr.as_ion() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn >= *prn;
                            }
                        }
                        pass
                    } else if let Some((_, sv, _)) = fr.as_eop() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn >= *prn;
                            }
                        }
                        pass
                    } else if let Some((_, sv, _)) = fr.as_sto() {
                        let mut pass = false;
                        for (constell, prn) in &filter {
                            if *constell == sv.constellation {
                                pass |= sv.prn >= *prn;
                            }
                        }
                        pass
                    } else {
                        // non existing
                        false
                    }
                });
                frames.len() > 0
            });
        },
        _ => {}, // Other items: either not supported, or do not apply
    }
}

#[cfg(feature = "processing")]
impl Mask for Record {
    fn mask(&self, mask: MaskFilter) -> Self {
        let mut s = self.clone();
        s.mask_mut(mask);
        s
    }
    fn mask_mut(&mut self, mask: MaskFilter) {
        match mask.operand {
            MaskOperand::Equals => mask_mut_equal(self, mask.item),
            MaskOperand::NotEquals => mask_mut_ineq(self, mask.item),
            MaskOperand::GreaterThan => mask_mut_gt(self, mask.item),
            MaskOperand::GreaterEquals => mask_mut_geq(self, mask.item),
            MaskOperand::LowerThan => mask_mut_lt(self, mask.item),
            MaskOperand::LowerEquals => mask_mut_leq(self, mask.item),
        }
    }
}

/*
 * Decimates only a given record subset
 */
#[cfg(feature = "processing")]
fn decimate_data_subset(record: &mut Record, subset: &Record, target: &TargetItem) {
    match target {
        TargetItem::SvItem(svs) => {
            /*
             * remove Sv data that should now be missing
             */
            for (epoch, frames) in record.iter_mut() {
                if subset.get(epoch).is_none() {
                    // for specified targets, this should now be removed
                    frames.retain(|fr| {
                        if let Some((_, sv, _)) = fr.as_eph() {
                            svs.contains(sv)
                        } else if let Some((_, sv, _)) = fr.as_ion() {
                            svs.contains(sv)
                        } else if let Some((_, sv, _)) = fr.as_sto() {
                            svs.contains(sv)
                        } else if let Some((_, sv, _)) = fr.as_eop() {
                            svs.contains(sv)
                        } else {
                            false // all cases already covered
                        }
                    });
                }
            }
        },
        TargetItem::ConstellationItem(constells_list) => {
            /*
             * Removes ephemeris frames that should now be missing
             */
            for (epoch, frames) in record.iter_mut() {
                if subset.get(epoch).is_none() {
                    // for specified targets, this should now be removed
                    frames.retain(|fr| {
                        if let Some((_, sv, _)) = fr.as_eph() {
                            constells_list.contains(&sv.constellation)
                        } else if let Some((_, sv, _)) = fr.as_ion() {
                            constells_list.contains(&sv.constellation)
                        } else if let Some((_, sv, _)) = fr.as_sto() {
                            constells_list.contains(&sv.constellation)
                        } else if let Some((_, sv, _)) = fr.as_eop() {
                            constells_list.contains(&sv.constellation)
                        } else {
                            false // all cases already covered
                        }
                    });
                }
            }
        },
        TargetItem::OrbitItem(_orbit_fields) => {
            /*
             * Removes ephemeris frames that should now be missing
             */
            unimplemented!("specific orbit field decimation");
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

#[cfg(feature = "processing")]
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

#[cfg(feature = "processing")]
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

#[cfg(feature = "processing")]
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
