use bitflags::bitflags;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use thiserror::Error;

use crate::{
    algorithm::Decimation, constellation, epoch, gnss_time::TimeScaling, merge, merge::Merge,
	Carrier,
    prelude::*, split, split::Split, sv, types::Type, version::Version, Observable,
	processing::{
		TargetItem,
		Preprocessing, 
		Filter, MaskOperand, 
		SmoothingFilter,
		Combination, Combine,
		Processing,
		IonoDelayDetector,
	},
};

use hifitime::Duration;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse epoch")]
    EpochError(#[from] epoch::Error),
    #[error("failed to identify constellation")]
    ConstellationError(#[from] constellation::Error),
    #[error("failed to parse sv")]
    SvError(#[from] sv::Error),
    #[error("failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse vehicules properly (nb_sat mismatch)")]
    EpochParsingError,
    #[error("line is empty")]
    MissingData,
}

#[cfg(feature = "serde")]
use serde::Serialize;

/// `Ssi` describes signals strength
#[repr(u8)]
#[derive(PartialOrd, Ord, PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Ssi {
    /// Ssi ~= 0 dB/Hz
    DbHz0 = 0,
    /// Ssi < 12 dB/Hz
    DbHz12 = 1,
    /// 12 dB/Hz <= Ssi < 17 dB/Hz
    DbHz12_17 = 2,
    /// 18 dB/Hz <= Ssi < 23 dB/Hz
    DbHz18_23 = 3,
    /// 24 dB/Hz <= Ssi < 29 dB/Hz
    DbHz24_29 = 4,
    /// 30 dB/Hz <= Ssi < 35 dB/Hz
    DbHz30_35 = 5,
    /// 36 dB/Hz <= Ssi < 41 dB/Hz
    DbHz36_41 = 6,
    /// 42 dB/Hz <= Ssi < 47 dB/Hz
    DbHz42_47 = 7,
    /// 48 dB/Hz <= Ssi < 53 dB/Hz
    DbHz48_53 = 8,
    /// Ssi >= 54 dB/Hz
    DbHz54 = 9,
}

impl std::fmt::Display for Ssi {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::DbHz0 => "0".fmt(f),
            Self::DbHz12 => "1".fmt(f),
            Self::DbHz12_17 => "2".fmt(f),
            Self::DbHz18_23 => "3".fmt(f),
            Self::DbHz24_29 => "4".fmt(f),
            Self::DbHz30_35 => "5".fmt(f),
            Self::DbHz36_41 => "6".fmt(f),
            Self::DbHz42_47 => "7".fmt(f),
            Self::DbHz48_53 => "8".fmt(f),
            Self::DbHz54 => "9".fmt(f),
        }
    }
}

impl Default for Ssi {
    fn default() -> Ssi {
        Ssi::DbHz54
    }
}

impl FromStr for Ssi {
    type Err = std::io::Error;
    fn from_str(code: &str) -> Result<Self, Self::Err> {
        match code {
            "0" => Ok(Ssi::DbHz0),
            "1" => Ok(Ssi::DbHz12),
            "2" => Ok(Ssi::DbHz12_17),
            "3" => Ok(Ssi::DbHz18_23),
            "4" => Ok(Ssi::DbHz24_29),
            "5" => Ok(Ssi::DbHz30_35),
            "6" => Ok(Ssi::DbHz36_41),
            "7" => Ok(Ssi::DbHz42_47),
            "8" => Ok(Ssi::DbHz48_53),
            "9" => Ok(Ssi::DbHz54),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid Ssi code",
            )),
        }
    }
}

impl Ssi {
    /// Returns true if `self` is a bad signal level, very poor quality,
    /// measurements should be discarded
    pub fn is_bad(self) -> bool {
        self <= Ssi::DbHz18_23
    }
    /// Returns true if `self` is a weak signal level, poor quality
    pub fn is_weak(self) -> bool {
        self < Ssi::DbHz30_35
    }
    /// Returns true if `self` is a strong signal level, good quality as defined by standard
    pub fn is_strong(self) -> bool {
        self >= Ssi::DbHz30_35
    }
    /// Returns true if `self` is a very strong signal level, very high quality
    pub fn is_excellent(self) -> bool {
        self > Ssi::DbHz42_47
    }
    /// Returns true if `self` matches a strong signal level (defined by standard)
    pub fn is_ok(self) -> bool {
        self.is_strong()
    }
}

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct LliFlags: u8 {
        /// Current epoch is marked Ok or Unknown status
        const OK_OR_UNKNOWN = 0x00;
        /// Lock lost between previous observation and current observation,
        /// cycle slip is possible
        const LOCK_LOSS = 0x01;
        /// Half cycle slip marker
        const HALF_CYCLE_SLIP = 0x02;
        /// Observing under anti spoofing,
        /// might suffer from decreased SNR - decreased signal quality
        const UNDER_ANTI_SPOOFING = 0x04;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ObservationData {
    /// physical measurement
    pub obs: f64,
    /// Lock loss indicator
    pub lli: Option<LliFlags>,
    /// Signal strength indicator
    pub ssi: Option<Ssi>,
}

impl std::ops::Add for ObservationData {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            lli: self.lli,
            ssi: self.ssi,
            obs: self.obs + rhs.obs,
        }
    }
}

impl std::ops::AddAssign for ObservationData {
    fn add_assign(&mut self, rhs: Self) {
        self.obs += rhs.obs;
    }
}

impl ObservationData {
    /// Builds new ObservationData structure from given predicates
    pub fn new(obs: f64, lli: Option<LliFlags>, ssi: Option<Ssi>) -> ObservationData {
        ObservationData { obs, lli, ssi }
    }
    /// Returns `true` if self is determined as `ok`.    
    /// self is declared `ok` if LLI and SSI flags are not provided,
    /// because they are considered as unknown/ok if missing by default.   
    /// If LLI exists:    
    ///    + LLI must match the LliFlags::OkOrUnknown flag (strictly)    
    /// if SSI exists:    
    ///    + SSI must match the .is_ok() criteria, refer to API
    pub fn is_ok(self) -> bool {
        let lli_ok = self.lli.unwrap_or(LliFlags::OK_OR_UNKNOWN) == LliFlags::OK_OR_UNKNOWN;
        let ssi_ok = self.ssi.unwrap_or(Ssi::default()).is_ok();
        lli_ok && ssi_ok
    }

    /// Returns Real Distance, by converting observed pseudo range,
    /// and compensating for distant and local clock offsets.
    /// See [p17-p18 of the RINEX specifications]. It makes only
    /// sense to apply this method on Pseudo Range observations.
    /// - rcvr_offset: receiver clock offset for this epoch, given in file
    /// - sv_offset: sv clock offset
    /// - bias: other (optionnal..) additive biases
    pub fn pr_real_distance(&self, rcvr_offset: f64, sv_offset: f64, biases: f64) -> f64 {
        self.obs + 299_792_458.0_f64 * (rcvr_offset - sv_offset) + biases
    }
}

/// Observation Record content.
/// Measurements are sorted by [hifitime::Epoch],
/// but unlike other RINEX records, a [epoch::EpochFlag] is associated to it.
/// An epoch possibly comprises the receiver clock offset
/// and a list of physical measurements, sorted by Space vehicule and observable.
/// Phase data is offset so they start at 0 (null initial phase).
/// ```
/// use rinex::*;
/// // grab a CRINEX (compressed OBS RINEX)
/// let rnx = Rinex::from_file("../test_resources/CRNX/V3/KUNZ00CZE.crx")
///    .unwrap();
/// // grab record
/// let record = rnx.record.as_obs()
///    .unwrap();
/// // browse epochs
/// for (epoch, (clock_offset, vehicules)) in record.iter() {
///    if let Some(clock_offset) = clock_offset {
///        // got clock offset @ given epoch
///    }
///    for (vehicule, observables) in vehicules.iter() {
///        for (observable, observation) in observables.iter() {
///            /// `observable` is a standard 3 letter string code
///            /// main measurement is `observation.data` (f64)
///            if let Some(lli) = observation.lli {
///                // Sometimes observations have an LLI flag attached to them,
///                // implemented in the form of a `bitflag`,
///                // for convenient binary masking
///
///            }
///            if let Some(ssii) = observation.ssi {
///                // Sometimes observations come with an SSI indicator
///            }
///        }
///    }
/// }
/// ```
pub type Record = BTreeMap<
    (Epoch, EpochFlag),
    (
        Option<f64>,
        BTreeMap<sv::Sv, HashMap<Observable, ObservationData>>,
    ),
>;

/// Returns true if given content matches a new OBSERVATION data epoch
pub(crate) fn is_new_epoch(line: &str, v: Version) -> bool {
    if v.major < 3 {
        if line.len() < 30 {
            false
        } else {
            epoch::parse(&line[0..29]).is_ok()
        }
    } else {
        // Modern RINEX
        // OBS::V3 behaves like all::V4
        match line.chars().nth(0) {
            Some(c) => {
                c == '>' // epochs always delimited
                         // by this new identifier
            },
            _ => false,
        }
    }
}

/// Builds `Record` entry for `ObservationData`
/// from given epoch content
pub(crate) fn parse_epoch(
    header: &Header,
    content: &str,
) -> Result<
    (
        (Epoch, EpochFlag),
        Option<f64>,
        BTreeMap<sv::Sv, HashMap<Observable, ObservationData>>,
    ),
    Error,
> {
    let mut lines = content.lines();
    let mut line = match lines.next() {
        Some(l) => l,
        _ => return Err(Error::MissingData),
    };

    // epoch::
    let mut offset: usize = 2+1 // Y
        +2+1 // d
        +2+1 // m
        +2+1 // h
        +2+1 // m
        +11; // secs

    // V > 2 epoch::year is a 4 digit number
    if header.version.major > 2 {
        offset += 2
    }

    // V > 2 might start with a ">" marker
    if line.starts_with(">") {
        line = line.split_at(1).1.clone();
    }

    let (date, rem) = line.split_at(offset + 3);
    let (n_sat, rem) = rem.split_at(3);
    let n_sat = u16::from_str_radix(n_sat.trim(), 10)?;
    let epoch = epoch::parse(date)?;

    // previously identified observables (that we expect)
    let obs = header.obs.as_ref().unwrap();
    let observables = &obs.codes;

    // grab possible clock offset
    let offs: Option<&str> = match header.version.major < 2 {
        true => {
            // RINEX 2
            // clock offsets are last 12 characters
            if line.len() > 60 - 12 {
                Some(line.split_at(60 - 12).1.trim())
            } else {
                None
            }
        },
        false => {
            // RINEX 3
            let min_len: usize = 4+1 // y
                +2+1 // m
                +2+1 // d
                +2+1 // h
                +2+1 // m
                +11+1// s
                +3   // flag
                +3; // n_sat
            if line.len() > min_len {
                // RINEX3: clock offset precision was increased
                Some(line.split_at(min_len).1.trim()) // this handles it naturally
            } else {
                None
            }
        },
    };
    let clock_offset: Option<f64> = match offs.is_some() {
        true => {
            if let Ok(f) = f64::from_str(offs.unwrap()) {
                Some(f)
            } else {
                None // parsing failed for some reason
            }
        },
        false => None, // empty field
    };

    let data = match header.version.major {
        2 => {
            // grab system descriptions
            //  current line remainder
            //  and possible following lines
            // This remains empty on RINEX3, because we have such information
            // on following lines, which is much more convenient
            let mut systems = String::with_capacity(24 * 3); //SVNN
            systems.push_str(rem.trim());
            while systems.len() / 3 < n_sat.into() {
                if let Some(l) = lines.next() {
                    systems.push_str(l.trim());
                } else {
                    return Err(Error::MissingData);
                }
            }
            parse_v2(&header, &systems, observables, lines)
        },
        _ => parse_v3(observables, lines),
    };
    Ok((epoch, clock_offset, data))
}

/*
 * Parses a V2 epoch from given lines iteratoor
 * Vehicule description is contained in the epoch descriptor
 * Each vehicule content is wrapped into several lines
 */
fn parse_v2(
    header: &Header,
    systems: &str,
    header_observables: &HashMap<Constellation, Vec<Observable>>,
    lines: std::str::Lines<'_>,
) -> BTreeMap<Sv, HashMap<Observable, ObservationData>> {
    let svnn_size = 3; // SVNN standard
    let nb_max_observables = 5; // in a single line
    let observable_width = 16; // data + 2 flags + 1 whitespace
    let mut sv_ptr = 0; // svnn pointer
    let mut obs_ptr = 0; // observable pointer
    let mut data: BTreeMap<Sv, HashMap<Observable, ObservationData>> = BTreeMap::new();
    let mut inner: HashMap<Observable, ObservationData> = HashMap::with_capacity(5);
    let mut sv: Sv;
    let mut observables: &Vec<Observable>;
    //println!("SYSTEMS \"{}\"", systems); // DEBUG

    // parse first system we're dealing with
    if systems.len() < svnn_size {
        // Can't even parse a single vehicule;
        // epoch descriptor is totally corrupt, stop here
        return data;
    }

    /*
     * identify 1st system
     */
    let max = std::cmp::min(svnn_size, systems.len()); // covers epoch with a unique vehicule
    let system = &systems[0..max];

    if let Ok(ssv) = Sv::from_str(system) {
        sv = ssv;
    } else {
        // mono constellation context
        if let Ok(prn) = u8::from_str_radix(system.trim(), 10) {
            if let Some(constellation) = header.constellation {
                sv = Sv { prn, constellation }
            } else {
                panic!("faulty RINEX2 constellation /sv definition");
            }
        } else {
            // can't parse 1st vehicule
            return data;
        }
    }
    sv_ptr += svnn_size; // increment pointer
                         // grab observables for this vehicule
    if let Some(o) = header_observables.get(&sv.constellation) {
        observables = &o;
    } else {
        // failed to identify observations for this vehicule
        return data;
    }

    for line in lines {
        // browse all lines provided
        //println!("parse_v2: \"{}\"", line); //DEBUG
        let line_width = line.len();
        if line_width < 10 {
            //println!("\nEMPTY LINE: \"{}\"", line); //DEBUG
            // line is empty
            // add maximal amount of vehicules possible
            obs_ptr += std::cmp::min(nb_max_observables, observables.len() - obs_ptr);
            // nothing to parse
        } else {
            // not a empty line
            //println!("\nLINE: \"{}\"", line); //DEBUG
            let nb_obs = num_integer::div_ceil(line_width, observable_width); // nb observations in this line
                                                                              //println!("NB OBS: {}", nb_obs); //DEBUG
                                                                              // parse all obs
            for i in 0..nb_obs {
                obs_ptr += 1;
                if obs_ptr > observables.len() {
                    // line is abnormally long compared to header definitions
                    //  parsing would fail
                    break;
                }
                let slice: &str = match i {
                    0 => {
                        &line[0..std::cmp::min(17, line_width)] // manage trimmed single obs
                    },
                    _ => {
                        let start = i * observable_width;
                        let end = std::cmp::min((i + 1) * observable_width, line_width); // trimmed lines
                        &line[start..end]
                    },
                };
                //println!("WORK CONTENT \"{}\"", slice); //DEBUG
                //TODO: improve please
                let obs = &slice[0..std::cmp::min(slice.len(), 14)]; // trimmed observations
                                                                     //println!("OBS \"{}\"", obs); //DEBUG
                let mut lli: Option<LliFlags> = None;
                let mut ssi: Option<Ssi> = None;
                if let Ok(obs) = f64::from_str(obs.trim()) {
                    // parse obs
                    if slice.len() > 14 {
                        let lli_str = &slice[14..15];
                        if let Ok(u) = u8::from_str_radix(lli_str, 10) {
                            lli = LliFlags::from_bits(u);
                        }
                        if slice.len() > 15 {
                            let ssi_str = &slice[15..16];
                            if let Ok(s) = Ssi::from_str(ssi_str) {
                                ssi = Some(s);
                            }
                        }
                    }
                    //println!("{} {:?} {:?} ==> {}", obs, lli, ssi, obscodes[obs_ptr-1]); //DEBUG
                    inner.insert(
                        observables[obs_ptr - 1].clone(),
                        ObservationData { obs, lli, ssi },
                    );
                } //f64::obs
            } // parsing all observations
            if nb_obs < nb_max_observables {
                obs_ptr += nb_max_observables - nb_obs;
            }
        }
        //println!("OBS COUNT {}", obs_ptr); //DEBUG

        if obs_ptr >= observables.len() {
            // we're done with current vehicule
            // build data
            data.insert(sv, inner.clone());
            inner.clear(); // prepare for next vehicule
            obs_ptr = 0;
            //identify next vehicule
            if sv_ptr >= systems.len() {
                // last vehicule
                return data;
            }
            // identify next vehicule
            let start = sv_ptr;
            let end = std::cmp::min(sv_ptr + svnn_size, systems.len()); // trimed epoch description
            let system = &systems[start..end];
            //println!("NEW SYSTEM \"{}\"\n", system); //DEBUG
            if let Ok(ssv) = Sv::from_str(system) {
                sv = ssv;
            } else {
                // mono constellation context
                if let Ok(prn) = u8::from_str_radix(system.trim(), 10) {
                    if let Some(constellation) = header.constellation {
                        sv = Sv { prn, constellation }
                    } else {
                        panic!("faulty RINEX2 constellation /sv definition");
                    }
                } else {
                    // can't parse vehicule
                    return data;
                }
            }
            sv_ptr += svnn_size; // increment pointer
                                 // grab observables for this vehicule
            if let Some(o) = header_observables.get(&sv.constellation) {
                observables = &o;
            } else {
                // failed to identify observations for this vehicule
                return data;
            }
        }
    } // for all lines provided
    data
}

/*
 * Parses a V3 epoch from given lines iteratoor
 * Format is much simpler, one vehicule is described in a single line
 */
fn parse_v3(
    observables: &HashMap<Constellation, Vec<Observable>>,
    lines: std::str::Lines<'_>,
) -> BTreeMap<Sv, HashMap<Observable, ObservationData>> {
    let svnn_size = 3; // SVNN standard
    let observable_width = 16; // data + 2 flags
    let mut data: BTreeMap<Sv, HashMap<Observable, ObservationData>> = BTreeMap::new();
    let mut inner: HashMap<Observable, ObservationData> = HashMap::with_capacity(5);
    for line in lines {
        // browse all lines
        //println!("parse_v3: \"{}\"", line); //DEBUG
        let (sv, line) = line.split_at(svnn_size);
        if let Ok(sv) = Sv::from_str(sv) {
            //println!("SV: \"{}\"", sv); //DEBUG
            if let Some(obscodes) = observables.get(&sv.constellation) {
                let nb_obs = line.len() / observable_width;
                inner.clear();
                //println!("NB OBS: {}", nb_obs); //DEBUG
                let mut rem = line;
                for i in 0..nb_obs {
                    if i == obscodes.len() {
                        break; // line abnormally long
                               // does not match previous Header definitions
                               // => would not be able to sort data
                    }
                    let split_offset = std::cmp::min(observable_width, rem.len()); // avoid overflow on last obs
                    let (content, r) = rem.split_at(split_offset);
                    //println!("content \"{}\" \"{}\"", content, r); //DEBUG
                    rem = r.clone();
                    let content_len = content.len();
                    let mut ssi: Option<Ssi> = None;
                    let mut lli: Option<LliFlags> = None;
                    let obs = &content[0..std::cmp::min(observable_width - 2, content_len)];
                    //println!("OBS \"{}\"", obs); //DEBUG
                    if let Ok(obs) = f64::from_str(obs.trim()) {
                        if content_len > observable_width - 2 {
                            let lli_str = &content[observable_width - 2..observable_width - 1];
                            if let Ok(u) = u8::from_str_radix(lli_str, 10) {
                                lli = LliFlags::from_bits(u);
                            }
                        }
                        if content_len > observable_width - 1 {
                            let ssi_str = &content[observable_width - 1..observable_width];
                            if let Ok(s) = Ssi::from_str(ssi_str) {
                                ssi = Some(s);
                            }
                        }
                        //println!("LLI {:?}", lli); //DEBUG
                        //println!("SSI {:?}", ssi);
                        // build content
                        inner.insert(obscodes[i].clone(), ObservationData { obs, lli, ssi });
                    }
                }
                if inner.len() > 0 {
                    data.insert(sv, inner.clone());
                }
            } //got some observables to work with
        } // Sv::from_str failed()
    } //browse all lines
    data
}

/// Formats one epoch according to standard definitions
pub(crate) fn fmt_epoch(
    epoch: Epoch,
    flag: EpochFlag,
    clock_offset: &Option<f64>,
    data: &BTreeMap<Sv, HashMap<Observable, ObservationData>>,
    header: &Header,
) -> String {
    if header.version.major < 3 {
        fmt_epoch_v2(epoch, flag, clock_offset, data, header)
    } else {
        fmt_epoch_v3(epoch, flag, clock_offset, data, header)
    }
}

fn fmt_epoch_v3(
    epoch: Epoch,
    flag: EpochFlag,
    clock_offset: &Option<f64>,
    data: &BTreeMap<Sv, HashMap<Observable, ObservationData>>,
    header: &Header,
) -> String {
    let mut lines = String::with_capacity(128);
    let observables = &header.obs.as_ref().unwrap().codes;

    lines.push_str(&format!(
        "> {} {:2}",
        epoch::format(epoch, Some(flag), Type::ObservationData, 3),
        data.len()
    ));

    if let Some(data) = clock_offset {
        lines.push_str(&format!("{:13.4}", data));
    }

    lines.push_str("\n");
    for (sv, data) in data.iter() {
        lines.push_str(&format!("{}", sv.to_string()));
        if let Some(observables) = observables.get(&sv.constellation) {
            for observable in observables {
                if let Some(observation) = data.get(observable) {
                    lines.push_str(&format!("{:14.3}", observation.obs));
                    if let Some(flag) = observation.lli {
                        lines.push_str(&format!("{}", flag.bits()));
                    } else {
                        lines.push_str(" ");
                    }
                    if let Some(flag) = observation.ssi {
                        lines.push_str(&format!("{}", flag));
                    } else {
                        lines.push_str(" ");
                    }
                } else {
                    lines.push_str(&format!("                "));
                }
            }
        }
        lines.push_str("\n");
    }
    lines
}

fn fmt_epoch_v2(
    epoch: Epoch,
    flag: EpochFlag,
    clock_offset: &Option<f64>,
    data: &BTreeMap<Sv, HashMap<Observable, ObservationData>>,
    header: &Header,
) -> String {
    let mut lines = String::with_capacity(128);
    let observables = &header.obs.as_ref().unwrap().codes;

    lines.push_str(&format!(
        " {} {:2}",
        epoch::format(epoch, Some(flag), Type::ObservationData, 2),
        data.len()
    ));

    let mut index = 0_u8;
    for (sv_index, (sv, _)) in data.iter().enumerate() {
        if index == 12 {
            index = 0;
            if sv_index == 12 {
                // first line
                if let Some(data) = clock_offset {
                    // push clock offsets
                    lines.push_str(&format!(" {:9.1}", data));
                }
            }
            lines.push_str(&format!("\n                                "));
        }
        lines.push_str(&sv.to_string());
        index += 1;
    }
    let obs_per_line = 5;
    // for each vehicule per epoch
    for (sv, observations) in data.iter() {
        // follow list of observables, as described in header section
        // for given constellation
        if let Some(observables) = observables.get(&sv.constellation) {
            for (obs_index, observable) in observables.iter().enumerate() {
                if obs_index % obs_per_line == 0 {
                    lines.push_str("\n");
                }
                if let Some(observation) = observations.get(observable) {
                    let formatted_obs = format!("{:14.3}", observation.obs);
                    let formatted_flags: String = match observation.lli {
                        Some(lli) => match observation.ssi {
                            Some(ssi) => format!("{}{}", lli.bits(), ssi),
                            _ => format!("{} ", lli.bits()),
                        },
                        _ => match observation.ssi {
                            Some(ssi) => format!(" {}", ssi),
                            _ => "  ".to_string(),
                        },
                    };
                    lines.push_str(&formatted_obs);
                    lines.push_str(&formatted_flags);
                } else {
                    // --> data is not provided: BLANK
                    lines.push_str("                ");
                }
            }
        }
    }
    lines.push_str("\n");
    lines
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
        for (epoch, (clk_offset, vehicules)) in rhs.iter() {
            if let Some((cclk_offset, vvehicules)) = self.get_mut(epoch) {
                if let Some(clk_offset) = clk_offset {
                    if cclk_offset.is_none() {
                        *cclk_offset = Some(*clk_offset); // clock offset is now provided
                    }
                }
                for (vehicule, observations) in vehicules.iter() {
                    if let Some(oobservations) = vvehicules.get_mut(vehicule) {
                        for (observable, data) in observations.iter() {
                            if let Some(ddata) = oobservations.get_mut(observable) {
                                // observation both provided
                                //  fill missing flags but leave data untouched
                                if let Some(lli) = data.lli {
                                    if ddata.lli.is_none() {
                                        ddata.lli = Some(lli);
                                    }
                                }
                                if let Some(ssi) = data.ssi {
                                    if ddata.ssi.is_none() {
                                        ddata.ssi = Some(ssi);
                                    }
                                }
                            } else {
                                //new observation
                                oobservations.insert(observable.clone(), data.clone());
                            }
                        }
                    } else {
                        // new vehicule
                        vvehicules.insert(*vehicule, observations.clone());
                    }
                }
            } else {
                // new epoch
                self.insert(*epoch, (*clk_offset, vehicules.clone()));
            }
        }
        Ok(())
    }
}

impl Split<Record> for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let r0 = self
            .iter()
            .flat_map(|(k, v)| {
                if k.0 < epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self
            .iter()
            .flat_map(|(k, v)| {
                if k.0 >= epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
		Ok((r0, r1))
    }
	fn split_dt(&self, duration: Duration) -> Result<Vec<Self>, split::Error> {
		let mut curr = Self::new();
		let mut ret: Vec<Self> = Vec::new();
		let mut prev: Option<Epoch> = None;
		for ((epoch, flag), data) in self {
			if let Some(mut prev) = prev {
				let dt = *epoch - prev;
				if dt >= duration {
					prev = *epoch;
					ret.push(curr);
					curr = Self::new();
				}
				curr.insert((*epoch, *flag), data.clone());
			} else {
				prev = Some(*epoch);
			}
		}
		Ok(ret)	
	}
}

impl Decimation<Record> for Record {
    /// Decimates Self by desired factor
    fn decim_by_ratio_mut(&mut self, r: u32) {
        let mut i = 0;
        self.retain(|_, _| {
            let retained = (i % r) == 0;
            i += 1;
            retained
        });
    }
    /// Copies and Decimates Self by desired factor
    fn decim_by_ratio(&self, r: u32) -> Self {
        let mut s = self.clone();
        s.decim_by_ratio_mut(r);
        s
    }
    /// Decimates Self to fit minimum epoch interval
    fn decim_by_interval_mut(&mut self, interval: Duration) {
        let mut last_retained: Option<Epoch> = None;
        self.retain(|(e, _), _| {
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
    /// Copies and Decimates Self to fit minimum epoch interval
    fn decim_by_interval(&self, interval: Duration) -> Self {
        let mut s = self.clone();
        s.decim_by_interval_mut(interval);
        s
    }
    fn decim_match_mut(&mut self, rhs: &Self) {
        self.retain(|e, _| rhs.get(e).is_some());
    }
    fn decim_match(&self, rhs: &Self) -> Self {
        let mut s = self.clone();
        s.decim_match_mut(&rhs);
        s
    }
}

impl TimeScaling<Record> for Record {
    fn convert_timescale(&mut self, ts: TimeScale) {
        self.iter_mut()
            .map(|((k, f), v)| ((k.in_time_scale(ts), f), v))
            .count();
    }
    fn with_timescale(&self, ts: TimeScale) -> Self {
        let mut s = self.clone();
        s.convert_timescale(ts);
        s
    }
}

fn smoothing_filter(record: &mut Record, filter: SmoothingFilter) {
	match filter {
		SmoothingFilter::HatchFilter => hatch_filter(record),
		SmoothingFilter::MovingAverage(Some(dt)) => {
		},
		SmoothingFilter::MovingAverage(None) => {
		},
	}
}
	
fn hatch_filter(rec: &mut Record) {
	let mut buffer: HashMap<Sv, HashMap<Observable, (u32, f64)>> = HashMap::new();
	for (_, (_, svs)) in rec {
		for (sv, observables) in svs {
			let rhs_observables = observables.clone();
			for (pr_observable, pr_observation) in observables {
				if !pr_observable.is_pseudorange_observable() {
					continue;
				}
				let pr_code = pr_observable.code()
					.unwrap();
				let ph_tolocate = "L".to_owned() + &pr_code;
				let mut ph_data: Option<f64> = None;
				for (rhs_observable, rhs_observation) in &rhs_observables { 
					let rhs_code = rhs_observable.to_string();
					if rhs_code == ph_tolocate {
						ph_data = Some(rhs_observation.obs);
						break;
					}
				}
				if let Some(ph_observation) = ph_data {
					if let Some(data) = buffer.get_mut(&sv) {
						if let Some((k, rms)) = data.get_mut(&pr_observable) {
							let x_k = pr_observation.obs - ph_observation;
							*rms = (x_k / *k as f64) + (((*k-1)/ *k) as f64) * *rms;
							*k += 1;
							pr_observation.obs = ph_observation - *rms;
						} else { // new observable
						}
					} else { // new sv
					}
				}
			}
		}
	}
}

impl Preprocessing for Record {
    fn filter(&self, filter: Filter) -> Self {
        let mut s = self.clone();
        s.filter_mut(filter);
        s
    }
    fn filter_mut(&mut self, filter: Filter) { 
		match filter {
			Filter::Smoothing(filter) => smoothing_filter(self, filter),
			Filter::Mask(mask) => {
				match mask.operand {
					MaskOperand::Equals => match mask.item {
						TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e == epoch),
						TargetItem::EpochFlagItem(flag) => self.retain(|(_, f), _| *f == flag),
						TargetItem::ConstellationItem(constells) => {
							self.retain(|_, (_, svs)| {
								svs.retain(|sv, _| constells.contains(&sv.constellation));
								svs.len() > 0
							});
						},
						TargetItem::SvItem(items) => {
							self.retain(|_, (_, svs)| {
								svs.retain(|sv, _| items.contains(&sv));
								svs.len() > 0
							});
						},
						TargetItem::ObservableItem(filter) => {
							self.retain(|_, (_, svs)| {
								svs.retain(|_, obs| {
									obs.retain(|code, _| filter.contains(&code));
									obs.len() > 0
								});
								svs.len() > 0
							});
						},
						_ => {},
					},
					MaskOperand::NotEquals => match mask.item {
						TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e != epoch),
						TargetItem::EpochFlagItem(flag) => self.retain(|(_, f), _| *f != flag),
						TargetItem::ConstellationItem(constells) => {
							self.retain(|_, (_, svs)| {
								svs.retain(|sv, _| !constells.contains(&sv.constellation));
								svs.len() > 0
							});
						},
						TargetItem::SvItem(items) => {
							self.retain(|_, (_, svs)| {
								svs.retain(|sv, _| !items.contains(&sv));
								svs.len() > 0
							});
						},
						TargetItem::ObservableItem(filter) => {
							self.retain(|_, (_, svs)| {
								svs.retain(|_, obs| {
									obs.retain(|code, _| !filter.contains(&code));
									obs.len() > 0
								});
								svs.len() > 0
							});
						},
						_ => {},
					},
					MaskOperand::GreaterEquals => match mask.item {
						TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e >= epoch),
						TargetItem::SvItem(items) => {
							self.retain(|_, (_, svs)| {
								svs.retain(|sv, _| {
									let mut retain = false;
									for item in &items {
										if item.constellation == sv.constellation {
											retain = sv.prn >= item.prn;
										} else {
											retain = true;
										}
									}
									retain
								});
								svs.len() > 0
							});
						},
						_ => {},
					},
					MaskOperand::GreaterThan => match mask.item {
						TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e > epoch),
						TargetItem::SvItem(items) => {
							self.retain(|_, (_, svs)| {
								svs.retain(|sv, _| {
									let mut retain = false;
									for item in &items {
										if item.constellation == sv.constellation {
											retain = sv.prn > item.prn;
										} else {
											retain = true;
										}
									}
									retain
								});
								svs.len() > 0
							});
						},
						_ => {},
					},
					MaskOperand::LowerEquals => match mask.item {
						TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e <= epoch),
						TargetItem::SvItem(items) => {
							self.retain(|_, (_, svs)| {
								svs.retain(|sv, _| {
									let mut retain = false;
									for item in &items {
										if item.constellation == sv.constellation {
											retain = sv.prn <= item.prn;
										} else {
											retain = true;
										}
									}
									retain
								});
								svs.len() > 0
							});
						},
						_ => {},
					},
					MaskOperand::LowerThan => match mask.item {
						TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e < epoch),
						TargetItem::SvItem(items) => {
							self.retain(|_, (_, svs)| {
								svs.retain(|sv, _| {
									let mut retain = false;
									for item in &items {
										if item.constellation == sv.constellation {
											retain = sv.prn < item.prn;
										} else {
											retain = true;
										}
									}
									retain
								});
								svs.len() > 0
							});
						},
						_ => {},
					},
				}
			},
			_ => {},
		}
	}
}

impl Processing for Record {
	fn min(&self) -> HashMap<Sv, HashMap<Observable, f64>> {
		let mut ret: HashMap<Sv, HashMap<Observable, f64>> = HashMap::new();
		for (_, (_, svs)) in self {
			for (sv, observables) in svs {
				for (observable, observation) in observables {
					if let Some(data) = ret.get_mut(sv) {
						if let Some(data) = data.get_mut(observable) {
							if observation.obs < *data {
								*data = observation.obs;
							}
						} else {
							data.insert(observable.clone(), observation.obs);
						}
					} else {
						let mut map: HashMap<Observable, f64> = HashMap::new();
						map.insert(observable.clone(), observation.obs);
						ret.insert(*sv, map);
					}
				}
			}
		}
		ret
	}
	fn min_observable(&self) -> HashMap<Observable, f64> {
		let mut ret: HashMap<Observable, f64> = HashMap::new();
		let min = self.min();
		for (sv, observables) in min {
			for (observable, minimum) in observables {
				if let Some(mmin) = ret.get_mut(&observable) {
					if minimum < *mmin {
						*mmin = minimum;
					}
				} else {
					ret.insert(observable.clone(), minimum);
				}
			}
		}
		ret
	}
	fn min_sv(&self) -> HashMap<Sv, f64> {
		let mut ret: HashMap<Sv, f64> = HashMap::new();
		let min = self.min();
		for (sv, observables) in min {
			for (observable, minimum) in observables {
				if let Some(mmin) = ret.get_mut(&sv) {
					if minimum < *mmin {
						*mmin = minimum;
					}
				} else {
					ret.insert(sv, minimum);
				}
			}
		}
		ret
	}
	fn max(&self) -> HashMap<Sv, HashMap<Observable, f64>> {
		let mut ret: HashMap<Sv, HashMap<Observable, f64>> = HashMap::new();
		for (_, (_, svs)) in self {
			for (sv, observables) in svs {
				for (observable, observation) in observables {
					if let Some(data) = ret.get_mut(sv) {
						if let Some(data) = data.get_mut(observable) {
							if observation.obs > *data {
								*data = observation.obs;
							}
						} else {
							data.insert(observable.clone(), observation.obs);
						}
					} else {
						let mut map: HashMap<Observable, f64> = HashMap::new();
						map.insert(observable.clone(), observation.obs);
						ret.insert(*sv, map);
					}
				}
			}
		}
		ret
	}
	fn max_observable(&self) -> HashMap<Observable, f64> {
		let mut ret: HashMap<Observable, f64> = HashMap::new();
		let max = self.max();
		for (sv, observables) in max {
			for (observable, maximum) in observables {
				if let Some(mmax) = ret.get_mut(&observable) {
					if maximum > *mmax {
						*mmax = maximum;
					}
				} else {
					ret.insert(observable.clone(), maximum);
				}
			}
		}
		ret
	}
	fn max_sv(&self) -> HashMap<Sv, f64> {
		let mut ret: HashMap<Sv, f64> = HashMap::new();
		let max = self.max();
		for (sv, observables) in max {
			for (observable, maximum) in observables {
				if let Some(mmax) = ret.get_mut(&sv) {
					if maximum < *mmax {
						*mmax = maximum;
					}
				} else {
					ret.insert(sv, maximum);
				}
			}
		}
		ret
	}
	fn mean(&self) -> HashMap<Sv, HashMap<Observable, f64>> {
		let mut sum: HashMap<Sv, HashMap<Observable, (u32, f64)>> = HashMap::new();
		for (_, (_, svs)) in self {
			for (sv, observables) in svs {
				for (observable, observation) in observables {
					if let Some(data) = sum.get_mut(sv) {
						if let Some((count, sum)) = data.get_mut(observable) {
							*count += 1;
							*sum += observation.obs;
						} else {
							data.insert(observable.clone(), (1, observation.obs));
						}
					} else {
						let mut map: HashMap<Observable, (u32, f64)> = HashMap::new();
						map.insert(observable.clone(), (1, observation.obs));
						sum.insert(*sv, map);
					}
				}
			}
		}
		let mut ret: HashMap<Sv, HashMap<Observable, f64>> = HashMap::new();
		for (sv, observables) in sum {
			for (observable, (count, sum)) in observables {
				if let Some(data) = ret.get_mut(&sv) {
					if let Some(data) = data.get_mut(&observable) {
						*data = sum / count as f64;
					} else {
						data.insert(observable.clone(), sum / count as f64);
					}
				} else {
					let mut map: HashMap<Observable, f64> = HashMap::new();
					map.insert(observable.clone(), sum / count as f64);
					ret.insert(sv, map);
				}
			}
		}
		ret
	}
	fn mean_observable(&self) -> HashMap<Observable, f64> {
		let mean = self.mean();
		let mut sum:  HashMap<Observable, (u32, f64)> = HashMap::new();
		for (sv, observables) in mean {
			for (observable, mean) in observables {
				if let Some((count, sum)) = sum.get_mut(&observable) {
					*count += 1;
					*sum += mean;
				} else {
					sum.insert(observable.clone(), (1, mean));
				}
			}
		}
		sum.iter()
			.map(|(k, (count, sum))| {
				(k.clone(), sum / *count as f64)
			})
			.collect()
	}
	fn mean_sv(&self) -> HashMap<Sv, f64> {
		let mean = self.mean();
		let mut sum:  HashMap<Sv, (u32, f64)> = HashMap::new();
		for (sv, observables) in mean {
			for (observable, mean) in observables {
				if let Some((count, sum)) = sum.get_mut(&sv) {
					*count += 1;
					*sum += mean;
				} else {
					sum.insert(sv.clone(), (1, mean));
				}
			}
		}
		sum.iter()
			.map(|(k, (count, sum))| {
				(k.clone(), sum / *count as f64)
			})
			.collect()
	}
	fn stddev(&self) -> HashMap<Sv, HashMap<Observable, f64>> {
		let mut stdvar = self.stdvar();
		for (_, observables) in stdvar.iter_mut() {
			for (observable, data) in observables.iter_mut() {
				*data = data.sqrt();
			}
		}
		stdvar
	}
	fn stddev_observable(&self) -> HashMap<Observable, f64> {
		let mut stdvar = self.stdvar_observable();
		for (_, data) in stdvar.iter_mut() {
			*data = data.sqrt();
		}
		stdvar
	}
	fn stddev_sv(&self) -> HashMap<Sv, f64> {
		let mut stdvar = self.stdvar_sv();
		for (_, data) in stdvar.iter_mut() {
			*data = data.sqrt();
		}
		stdvar
	}
	fn stdvar(&self) -> HashMap<Sv, HashMap<Observable, f64>> {
		let mean = self.mean();
		let mut ret: HashMap<Sv, HashMap<Observable, f64>> = HashMap::new();
		let mut diff: HashMap<Sv, HashMap<Observable, (u32, f64)>> = HashMap::new();
		for (_, (_, svs)) in self {
			for (sv, observables) in svs {
				for (observable, observation) in observables {
					let mean = mean.get(&sv)
						.unwrap()
							.get(&observable)
							.unwrap();
					if let Some(data) = diff.get_mut(sv) {
						if let Some((count, diff)) = data.get_mut(observable) {
							*count += 1;
							*diff += (observation.obs - mean).powf(2.0);
						} else {
							data.insert(observable.clone(), (1, (observation.obs - mean).powf(2.0))); 
						}
					} else {
						let mut map: HashMap<Observable, (u32, f64)> = HashMap::new();
						map.insert(observable.clone(), (1, (observation.obs - mean).powf(2.0))); 
						diff.insert(*sv, map);
					}
				}
			}
		}
		let mut ret: HashMap<Sv, HashMap<Observable, f64>> = HashMap::new();
		for (sv, observables) in diff {
			for (observable, (count, diff)) in observables {
				if let Some(data) = ret.get_mut(&sv) {
					if let Some(data) = data.get_mut(&observable) {
						*data = diff /  count as f64;
					} else {
						data.insert(observable.clone(), diff /  count as f64); 
					}
				} else {
					let mut map: HashMap<Observable, f64> = HashMap::new();
					map.insert(observable.clone(), diff /  count as f64); 
					ret.insert(sv, map);
				}
			}
		}
		ret
	}
	fn stdvar_sv(&self) -> HashMap<Sv, f64> {
		let mean = self.mean_sv();
		let mut diff: HashMap<Sv, (u32, f64)> = HashMap::new();
		for (_, (_, svs)) in self {
			for (sv, observables) in svs {
				for (observable, observation) in observables {
					let mean = mean.get(&sv)
						.unwrap();
					if let Some((count, diff)) = diff.get_mut(sv) {
						*count += 1;
						*diff += (observation.obs - mean).powf(2.0);
					} else {
						diff.insert(*sv, (1, (observation.obs - mean).powf(2.0)));
					}
				}
			}
		}
		diff.iter()
			.map(|(sv, (count, diff))| {
				(*sv, diff / *count as f64)
			})
			.collect()
	}
	fn stdvar_observable(&self) -> HashMap<Observable, f64> {
		let mean = self.mean_observable();
		let mut diff: HashMap<Observable, (u32, f64)> = HashMap::new();
		for (_, (_, svs)) in self {
			for (sv, observables) in svs {
				for (observable, observation) in observables {
					let mean = mean.get(&observable)
						.unwrap();
					if let Some((count, diff)) = diff.get_mut(observable) {
						*count += 1;
						*diff += (observation.obs - mean).powf(2.0);
					} else {
						diff.insert(observable.clone(), (1, (observation.obs - mean).powf(2.0)));
					}
				}
			}
		}
		diff.iter()
			.map(|(observable, (count, diff))| {
				(observable.clone(), diff / *count as f64)
			})
			.collect()
	}
	fn derivative(&self) -> BTreeMap<Epoch, HashMap<Sv, HashMap<Observable, f64>>> {
		let mut ret: BTreeMap<Epoch, HashMap<Sv, HashMap<Observable, f64>>> = BTreeMap::new();
		let mut prev: HashMap<Sv, HashMap<Observable, (Epoch, f64)>> = HashMap::new();
		for ((epoch, _), (_, svs)) in self {
			for (sv, observables) in svs {
				for (observable, observation) in observables {
					if let Some(prev) = prev.get_mut(&sv) {
						if let Some(prev) = prev.get_mut(&observable) {
							if let Some(data) = ret.get_mut(&epoch) {
								if let Some(data) = data.get_mut(&sv) {
									if let Some(data) = data.get_mut(&observable) {
										*data = (observation.obs - prev.1) / (*epoch - prev.0).to_unit(hifitime::Unit::Second);
									} else {
										data.insert(observable.clone(), (observation.obs - prev.1) / (*epoch - prev.0).to_unit(hifitime::Unit::Second));
									}
								} else {
									let mut map: HashMap<Observable, f64> = HashMap::new();
									map.insert(observable.clone(), (observation.obs - prev.1) / (*epoch - prev.0).to_unit(hifitime::Unit::Second));
									data.insert(*sv, map);
								}
							} else {
								let mut map: HashMap<Observable, f64> = HashMap::new();
								map.insert(observable.clone(), (observation.obs - prev.1) / (*epoch - prev.0).to_unit(hifitime::Unit::Second));
								let mut mmap: HashMap<Sv, HashMap<Observable, f64>> = HashMap::new();
								mmap.insert(*sv, map);
								ret.insert(*epoch, mmap);
							}
							*prev = (*epoch, observation.obs);
						} else {
							prev.insert(observable.clone(), (*epoch, observation.obs));
						}
					} else {
						let mut map: HashMap<Observable, (Epoch, f64)> = HashMap::new();
						map.insert(observable.clone(), (*epoch, observation.obs));
						prev.insert(*sv, map);
					}
				}
			}
		}
		ret
	}
}

/*
 * Forms all GF combinations
 */
fn gf_combination(record: &Record) -> HashMap<(Observable, Observable), HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
	let mut ret:  HashMap<(Observable, Observable), HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>
		= HashMap::new();
	for (epoch, (_, vehicules)) in record {
		for (sv, observations) in vehicules {
			for (lhs_observable, lhs_data) in observations {
				if !lhs_observable.is_phase_observable()
					&& !lhs_observable.is_pseudorange_observable()
				{
					continue; // only for these two physics
				}
				let lhs_code = lhs_observable.to_string();
				let lhs_carrier = &lhs_code[1..2];

				// determine another carrier
				let rhs_carrier = match lhs_carrier {
					// this will restrict combinations to
					"1" => "2", // 1 against 2
					_ => "1",   // M > 1 against 1
				};
				
				// locate a reference code against another carrier
				let mut reference: Option<(Observable, (f64, f64, f64))> = None;
				for (ref_observable, ref_data) in observations {
					let mut shared_physics = ref_observable.is_phase_observable()
						&& lhs_observable.is_phase_observable();
					shared_physics |= ref_observable.is_pseudorange_observable()
						&& lhs_observable.is_pseudorange_observable();
					if !shared_physics {
						continue;
					}

					let refcode = ref_observable.to_string();
					let carrier_code = &refcode[1..2];
					if carrier_code == rhs_carrier {
						if ref_observable.is_phase_observable() {
							let carrier = Carrier::from_code(sv.constellation, &refcode)
								.unwrap();
							reference = Some((ref_observable.clone(), (carrier.carrier_wavelength(), carrier.carrier_frequency_mhz(), ref_data.obs)));
						} else {
							reference = Some((ref_observable.clone(), (1.0, 1.0, ref_data.obs)));
						}
						break; // DONE searching
					}
				}

				if let Some((ref_observable, (ref_lambda, ref_freq, ref_data))) = reference { // got a reference
					let gf = match ref_observable.is_phase_observable() {
						true => {
							let carrier = Carrier::from_code(sv.constellation, &lhs_code)
								.unwrap();
							let lhs_lambda = carrier.carrier_wavelength();
							let lhs_freq = carrier.carrier_frequency_mhz();
							let gamma = lhs_freq / ref_freq;
							let total_scaling = 1.0 / (gamma.powf(2.0) - 1.0);
							(lhs_data.obs * lhs_lambda - ref_data * ref_lambda) * total_scaling 
						},
						false => ref_data - lhs_data.obs, // PR: sign differs
					};
					
					if let Some(data) = ret.get_mut(&(lhs_observable.clone(), ref_observable.clone())) {
						if let Some(data) = data.get_mut(&sv) {
							data.insert(*epoch, gf);
						} else {
							let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> =
								BTreeMap::new();
							bmap.insert(*epoch, gf);
							data.insert(*sv, bmap);
						}
					} else { // new combination
						let mut inject = true; // insert only if not already combined to some other signal 
						for ((lhs, rhs), _) in &ret {
							if lhs == lhs_observable {
								inject = false;
								break;
							}
							if rhs == lhs_observable {
								inject = false;
								break;
							}
						}
						if inject {
							let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> =
								BTreeMap::new();
							bmap.insert(*epoch, gf);
							let mut map: HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>> =
								HashMap::new();
							map.insert(*sv, bmap);
							ret.insert((lhs_observable.clone(), ref_observable), map);
						}
					}
				}
			}
		}
	}
	ret
}

/*
 * Forms all NL combinations
 */
fn nl_combination(record: &Record) -> HashMap<(Observable, Observable), HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
	let mut ret:  HashMap<(Observable, Observable), HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>
		= HashMap::new();
	for (epoch, (_, vehicules)) in record {
		for (sv, observations) in vehicules {
			for (lhs_observable, lhs_data) in observations {
				if !lhs_observable.is_phase_observable()
					&& !lhs_observable.is_pseudorange_observable()
				{
					continue; // only for these two physics
				}
				let lhs_code = lhs_observable.to_string();
				let lhs_carrier = &lhs_code[1..2];
				
				// determine another carrier
				let rhs_carrier = match lhs_carrier {
					// this will restrict combinations to
					"1" => "2", // 1 against 2
					_ => "1",   // M > 1 against 1
				};
				
				// locate a reference code against another carrier
				let mut reference: Option<(Observable, f64)> = None;
				for (ref_observable, ref_data) in observations {
					let mut shared_physics = ref_observable.is_phase_observable()
						&& lhs_observable.is_phase_observable();
					shared_physics |= ref_observable.is_pseudorange_observable()
						&& lhs_observable.is_pseudorange_observable();
					if !shared_physics {
						continue;
					}

					let refcode = ref_observable.to_string();
					let carrier_code = &refcode[1..2];
					if carrier_code == rhs_carrier {
						reference = Some((ref_observable.clone(), ref_data.obs));
						break; // DONE searching
					}
				}

				if let Some((ref_observable, ref_data)) = reference { // got a reference
					let gf = match ref_observable.is_phase_observable() {
						true => lhs_data.obs - ref_data, 
						false => ref_data - lhs_data.obs, // PR: sign differs
					};
					
					if let Some(data) = ret.get_mut(&(lhs_observable.clone(), ref_observable.clone())) {
						if let Some(data) = data.get_mut(&sv) {
							data.insert(*epoch, gf);
						} else {
							let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> =
								BTreeMap::new();
							bmap.insert(*epoch, gf);
							data.insert(*sv, bmap);
						}
					} else { // new combination
						let mut inject = true; // insert only if not already combined to some other signal 
						for ((lhs, rhs), _) in &ret {
							if lhs == lhs_observable {
								inject = false;
								break;
							}
							if rhs == lhs_observable {
								inject = false;
								break;
							}
						}
						if inject {
							let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> =
								BTreeMap::new();
							bmap.insert(*epoch, gf);
							let mut map: HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>> =
								HashMap::new();
							map.insert(*sv, bmap);
							ret.insert((lhs_observable.clone(), ref_observable), map);
						}
					}
				}
			}
		}
	}
	ret
}

/*
 * Forms all WL combinations
 */
fn wl_combination(record: &Record) -> HashMap<(Observable, Observable), HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
	let mut ret:  HashMap<(Observable, Observable), HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>
		= HashMap::new();
	for (epoch, (_, vehicules)) in record {
		for (sv, observations) in vehicules {
			for (lhs_observable, lhs_data) in observations {
				if !lhs_observable.is_phase_observable()
					&& !lhs_observable.is_pseudorange_observable()
				{
					continue; // only for these two physics
				}
				let lhs_code = lhs_observable.to_string();
				let lhs_carrier = &lhs_code[1..2];
				
				// determine another carrier
				let rhs_carrier = match lhs_carrier {
					// this will restrict combinations to
					"1" => "2", // 1 against 2
					_ => "1",   // M > 1 against 1
				};
				
				// locate a reference code against another carrier
				let mut reference: Option<(Observable, f64)> = None;
				for (ref_observable, ref_data) in observations {
					let mut shared_physics = ref_observable.is_phase_observable()
						&& lhs_observable.is_phase_observable();
					shared_physics |= ref_observable.is_pseudorange_observable()
						&& lhs_observable.is_pseudorange_observable();
					if !shared_physics {
						continue;
					}

					let refcode = ref_observable.to_string();
					let carrier_code = &refcode[1..2];
					if carrier_code == rhs_carrier {
						reference = Some((ref_observable.clone(), ref_data.obs));
						break; // DONE searching
					}
				}

				if let Some((ref_observable, ref_data)) = reference { // got a reference
					let gf = match ref_observable.is_phase_observable() {
						true => lhs_data.obs - ref_data, 
						false => ref_data - lhs_data.obs, // PR: sign differs
					};
					
					if let Some(data) = ret.get_mut(&(lhs_observable.clone(), ref_observable.clone())) {
						if let Some(data) = data.get_mut(&sv) {
							data.insert(*epoch, gf);
						} else {
							let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> =
								BTreeMap::new();
							bmap.insert(*epoch, gf);
							data.insert(*sv, bmap);
						}
					} else { // new combination
						let mut inject = true; // insert only if not already combined to some other signal 
						for ((lhs, rhs), _) in &ret {
							if lhs == lhs_observable {
								inject = false;
								break;
							}
							if rhs == lhs_observable {
								inject = false;
								break;
							}
						}
						if inject {
							let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> =
								BTreeMap::new();
							bmap.insert(*epoch, gf);
							let mut map: HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>> =
								HashMap::new();
							map.insert(*sv, bmap);
							ret.insert((lhs_observable.clone(), ref_observable), map);
						}
					}
				}
			}
		}
	}
	ret
}

/*
 * Forms all MW combinations
 */
fn mw_combination(record: &Record) -> HashMap<(Observable, Observable), HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
	let mut ret:  HashMap<(Observable, Observable), HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>
		= HashMap::new();
	for (epoch, (_, vehicules)) in record {
		for (sv, observations) in vehicules {
			for (lhs_observable, lhs_data) in observations {
				if !lhs_observable.is_phase_observable()
					&& !lhs_observable.is_pseudorange_observable()
				{
					continue; // only for these two physics
				}
				let lhs_code = lhs_observable.to_string();
				let lhs_carrier = &lhs_code[1..2];
				
				// determine another carrier
				let rhs_carrier = match lhs_carrier {
					// this will restrict combinations to
					"1" => "2", // 1 against 2
					_ => "1",   // M > 1 against 1
				};
				
				// locate a reference code against another carrier
				let mut reference: Option<(Observable, f64)> = None;
				for (ref_observable, ref_data) in observations {
					let mut shared_physics = ref_observable.is_phase_observable()
						&& lhs_observable.is_phase_observable();
					shared_physics |= ref_observable.is_pseudorange_observable()
						&& lhs_observable.is_pseudorange_observable();
					if !shared_physics {
						continue;
					}

					let refcode = ref_observable.to_string();
					let carrier_code = &refcode[1..2];
					if carrier_code == rhs_carrier {
						reference = Some((ref_observable.clone(), ref_data.obs));
						break; // DONE searching
					}
				}

				if let Some((ref_observable, ref_data)) = reference { // got a reference
					let gf = match ref_observable.is_phase_observable() {
						true => lhs_data.obs - ref_data, 
						false => ref_data - lhs_data.obs, // PR: sign differs
					};
					
					if let Some(data) = ret.get_mut(&(lhs_observable.clone(), ref_observable.clone())) {
						if let Some(data) = data.get_mut(&sv) {
							data.insert(*epoch, gf);
						} else {
							let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> =
								BTreeMap::new();
							bmap.insert(*epoch, gf);
							data.insert(*sv, bmap);
						}
					} else { // new combination
						let mut inject = true; // insert only if not already combined to some other signal 
						for ((lhs, rhs), _) in &ret {
							if lhs == lhs_observable {
								inject = false;
								break;
							}
							if rhs == lhs_observable {
								inject = false;
								break;
							}
						}
						if inject {
							let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> =
								BTreeMap::new();
							bmap.insert(*epoch, gf);
							let mut map: HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>> =
								HashMap::new();
							map.insert(*sv, bmap);
							ret.insert((lhs_observable.clone(), ref_observable), map);
						}
					}
				}
			}
		}
	}
	ret
}

impl Combine for Record {
	fn combine(&self, combination: Combination) -> HashMap<(Observable, Observable), HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
		match combination {
			Combination::GeometryFree => gf_combination(&self),
			Combination::NarrowLane => nl_combination(&self),
			Combination::WideLane => wl_combination(&self),
			Combination::MelbourneWubbena => mw_combination(&self),
		}
	}
}

use crate::{carrier, processing::Dcb};

impl Dcb for Record {
	fn dcb(&self) -> HashMap<String, HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
        let mut ret: HashMap<String, HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> =
            HashMap::new();
		for (epoch, (_, vehicules)) in self {
			for (sv, observations) in vehicules {
				for (lhs_observable, lhs_observation) in observations {
					if !lhs_observable.is_phase_observable() {
						if !lhs_observable.is_pseudorange_observable() {
							continue;
						}
					}
					let lhs_code = lhs_observable.to_string();
					let lhs_carrier = &lhs_code[1..2];
					let lhs_code = &lhs_code[1..];

					for rhs_code in carrier::KNOWN_CODES.iter() { // locate a reference code
						if *rhs_code != lhs_code { // code differs
							if rhs_code.starts_with(lhs_carrier) { // same carrier
								let tolocate = match lhs_observable.is_phase_observable() {
									true => "L".to_owned() + rhs_code, // same physics
									false => "C".to_owned() + rhs_code, // same physics
								};
								let tolocate = Observable::from_str(&tolocate)
									.unwrap();
								if let Some(rhs_observation) = observations.get(&tolocate) { // got a reference code
									let mut already_diffd = false;

									for (op, vehicules) in ret.iter_mut() {
										if op.contains(lhs_code) {
											already_diffd = true;

											// determine this code's role in the diff op
											// so it remains consistent
											let items: Vec<&str> = op.split("-").collect();
											
											if lhs_code == items[0] {
												// code is differenced
												if let Some(data) = vehicules.get_mut(&sv) {
													data.insert(
														*epoch,
														lhs_observation.obs - rhs_observation.obs,
													);
												} else {
													let mut bmap: BTreeMap<
														(Epoch, EpochFlag),
														f64,
													> = BTreeMap::new();
													bmap.insert(
														*epoch,
														lhs_observation.obs - rhs_observation.obs,
													);
													vehicules.insert(*sv, bmap);
												}
											} else { // code is refered to 
												if let Some(data) = vehicules.get_mut(&sv) {
													data.insert(
														*epoch,
														rhs_observation.obs - lhs_observation.obs,
													);
												} else {
													let mut bmap: BTreeMap<
														(Epoch, EpochFlag),
														f64,
													> = BTreeMap::new();
													bmap.insert(
														*epoch,
														rhs_observation.obs - lhs_observation.obs,
													);
													vehicules.insert(*sv, bmap);
												}
											}
										}
									}
									if !already_diffd {
										let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> =
											BTreeMap::new();
										bmap.insert(*epoch, lhs_observation.obs - rhs_observation.obs);
										let mut map: HashMap<
											Sv,
											BTreeMap<(Epoch, EpochFlag), f64>,
										> = HashMap::new();
										map.insert(*sv, bmap);
										ret.insert(format!("{}-{}", lhs_code, rhs_code), map);
									}
								}
							}
						}
					}
				}
			}
		}
		ret
	}
}

impl IonoDelayDetector for Record {
	fn iono_delay_detector(&self, max_dt: Duration) -> HashMap<Observable, HashMap<Sv, BTreeMap<Epoch, f64>>> {
		let gf = self.combine(Combination::GeometryFree);
		let mut ret:  HashMap<Observable, HashMap<Sv, BTreeMap<Epoch, f64>>>  = HashMap::new();
		let mut prev_data: HashMap<(Observable, Observable), HashMap<Sv, (Epoch, f64)>> = HashMap::new();
		for (combination, vehicles) in gf {
		let (lhs_observable, ref_observable) = combination.clone();
			if !ref_observable.is_phase_observable() {
				continue ; // only on phase data
			}
			for (sv, epochs) in vehicles {
				for ((epoch, _flag), data) in epochs {
					if let Some(prev) = prev_data.get_mut(&combination) {
						if let Some((prev_epoch, prev_data)) = prev.get_mut(&sv) {
							let dt = epoch - *prev_epoch;
							if dt <= max_dt {
								// accepted: push a new dy value
								let dy = data - *prev_data;
								if let Some(data) = ret.get_mut(&ref_observable) {
									if let Some(data) = data.get_mut(&sv) {
										data.insert(epoch, dy);
									} else {
										let mut bmap: BTreeMap<Epoch, f64> = BTreeMap::new();
										bmap.insert(epoch, dy);
										data.insert(sv, bmap);
									}
								} else {
									let mut bmap: BTreeMap<Epoch, f64> = BTreeMap::new();
									bmap.insert(epoch, dy);
									let mut map: HashMap<Sv, BTreeMap<Epoch, f64>> =
										HashMap::new();
									map.insert(sv, bmap);
									ret.insert(ref_observable.clone(), map);
								}
							}
							*prev_epoch = epoch;
							*prev_data = data;
						} else {
							prev.insert(sv, (epoch, data));
						}
					} else {
						let mut map: HashMap<Sv, (Epoch, f64)> = HashMap::new();
						map.insert(sv, (epoch, data));
						prev_data.insert(combination.clone(), map);
					}
				}
            }
        }
		ret
	}
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn ssi() {
        let ssi = Ssi::from_str("0").unwrap();
        assert_eq!(ssi, Ssi::DbHz0);
        assert_eq!(ssi.is_bad(), true);
        let ssi = Ssi::from_str("9").unwrap();
		assert_eq!(ssi.is_excellent(), true);
        let ssi = Ssi::from_str("10");
        assert_eq!(ssi.is_err(), true);
    }
    #[test]
    fn test_is_new_epoch() {
        assert_eq!(
            is_new_epoch(
                "95 01 01 00 00 00.0000000  0  7 06 17 21 22 23 28 31",
                Version { major: 2, minor: 0 }
            ),
            true
        );
        assert_eq!(
            is_new_epoch(
                "21700656.31447  16909599.97044          .00041  24479973.67844  24479975.23247",
                Version { major: 2, minor: 0 }
            ),
            false
        );
        assert_eq!(
            is_new_epoch(
                "95 01 01 11 00 00.0000000  0  8 04 16 18 19 22 24 27 29",
                Version { major: 2, minor: 0 }
            ),
            true
        );
        assert_eq!(
            is_new_epoch(
                "95 01 01 11 00 00.0000000  0  8 04 16 18 19 22 24 27 29",
                Version { major: 3, minor: 0 }
            ),
            false
        );
        assert_eq!(
            is_new_epoch(
                "> 2022 01 09 00 00 30.0000000  0 40",
                Version { major: 3, minor: 0 }
            ),
            true
        );
        assert_eq!(
            is_new_epoch(
                "> 2022 01 09 00 00 30.0000000  0 40",
                Version { major: 2, minor: 0 }
            ),
            false
        );
        assert_eq!(
            is_new_epoch(
                "G01  22331467.880   117352685.28208        48.950    22331469.28",
                Version { major: 3, minor: 0 }
            ),
            false
        );
    }
}
