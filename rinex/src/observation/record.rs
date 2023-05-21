use bitflags::bitflags;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use thiserror::Error;

use crate::{
    constellation, epoch,
    gnss_time::GnssTime,
    merge,
    merge::Merge,
    prelude::*,
    processing::{
        Combination, Combine, Decimate, DecimationType, Filter, Interpolate, IonoDelayDetector,
        Mask, MaskFilter, MaskOperand, Preprocessing, Processing, Smooth, SmoothingType,
        TargetItem,
    },
    split,
    split::Split,
    sv,
    types::Type,
    version::Version,
    Carrier, Observable,
};

use super::Snr;
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
    pub snr: Option<Snr>,
}

impl std::ops::Add for ObservationData {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            lli: self.lli,
            snr: self.snr,
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
    /// Builds new ObservationData structure
    pub fn new(obs: f64, lli: Option<LliFlags>, snr: Option<Snr>) -> ObservationData {
        ObservationData { obs, lli, snr }
    }
    /// Returns `true` if self is determined as `ok`.    
    /// Self is declared `ok` if LLI and SSI flags missing.
    /// If LLI exists:    
    ///    + LLI must match the LliFlags::OkOrUnknown flag (strictly)    
    /// if SSI exists:    
    ///    + SNR must match the .is_ok() criteria, refer to API
    pub fn is_ok(self) -> bool {
        let lli_ok = self.lli.unwrap_or(LliFlags::OK_OR_UNKNOWN) == LliFlags::OK_OR_UNKNOWN;
        let snr_ok = self.snr.unwrap_or(Snr::default()).strong();
        lli_ok && snr_ok
    }

    /// Returns true if self is considered Ok with respect to given
    /// SNR condition (>=)
    pub fn is_ok_snr(&self, min_snr: Snr) -> bool {
        if self
            .lli
            .unwrap_or(LliFlags::OK_OR_UNKNOWN)
            .intersects(LliFlags::OK_OR_UNKNOWN)
        {
            if let Some(snr) = self.snr {
                snr >= min_snr
            } else {
                false
            }
        } else {
            false
        }
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
///            if let Some(snri) = observation.snr {
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
        BTreeMap<Sv, HashMap<Observable, ObservationData>>,
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
                let mut snr: Option<Snr> = None;
                if let Ok(obs) = f64::from_str(obs.trim()) {
                    // parse obs
                    if slice.len() > 14 {
                        let lli_str = &slice[14..15];
                        if let Ok(u) = u8::from_str_radix(lli_str, 10) {
                            lli = LliFlags::from_bits(u);
                        }
                        if slice.len() > 15 {
                            let snr_str = &slice[15..16];
                            if let Ok(s) = Snr::from_str(snr_str) {
                                snr = Some(s);
                            }
                        }
                    }
                    //println!("{} {:?} {:?} ==> {}", obs, lli, snr, obscodes[obs_ptr-1]); //DEBUG
                    inner.insert(
                        observables[obs_ptr - 1].clone(),
                        ObservationData { obs, lli, snr },
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
                    let mut snr: Option<Snr> = None;
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
                            let snr_str = &content[observable_width - 1..observable_width];
                            if let Ok(s) = Snr::from_str(snr_str) {
                                snr = Some(s);
                            }
                        }
                        //println!("LLI {:?}", lli); //DEBUG
                        //println!("SSI {:?}", snr);
                        // build content
                        inner.insert(obscodes[i].clone(), ObservationData { obs, lli, snr });
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
                    if let Some(flag) = observation.snr {
                        lines.push_str(&format!("{:x}", flag));
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
                        Some(lli) => match observation.snr {
                            Some(snr) => format!("{}{:x}", lli.bits(), snr),
                            _ => format!("{} ", lli.bits()),
                        },
                        _ => match observation.snr {
                            Some(snr) => format!(" {:x}", snr),
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

impl Merge for Record {
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
                                if let Some(snr) = data.snr {
                                    if ddata.snr.is_none() {
                                        ddata.snr = Some(snr);
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

impl Split for Record {
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
            if let Some(p_epoch) = prev {
                let dt = *epoch - p_epoch;
                if dt >= duration {
                    prev = Some(*epoch);
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
    fn decimate_by_interval(&self, interval: Duration) -> Self {
        let mut s = self.clone();
        s.decimate_by_interval_mut(interval);
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

impl GnssTime for Record {
    fn timeseries(&self, dt: Duration) -> TimeSeries {
        let epochs: Vec<_> = self.keys().collect();
        TimeSeries::inclusive(
            epochs.get(0).expect("failed to determine first epoch").0,
            epochs
                .get(epochs.len() - 1)
                .expect("failed to determine last epoch")
                .0,
            dt,
        )
    }
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

impl Smooth for Record {
    /// Applies Hatch smoothing filter, returns smoothed Pseudo Range observations
    fn hatch_smoothing(&self, target: Option<TargetItem>) -> Self {
        let mut s = self.clone();
        s.hatch_smoothing_mut(target);
        s
    }
    /// Applies Hatch smoothing filter in place
    fn hatch_smoothing_mut(&mut self, target: Option<TargetItem>) {
        if let Some(item) = target {
            let mask = MaskFilter {
                item,
                operand: MaskOperand::Equals,
            };
            self.mask_mut(mask);
        }
        /*
         * smoothes pseudo range observations using special algorithm
         */
        let mut buffer: HashMap<Sv, HashMap<Observable, (u32, f64)>> = HashMap::new();
        for (_, (_, svs)) in self {
            for (sv, observables) in svs {
                let rhs_observables = observables.clone();
                for (pr_observable, pr_observation) in observables {
                    if !pr_observable.is_pseudorange_observable() {
                        continue;
                    }
                    let pr_code = pr_observable.code().unwrap();
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
                                *rms = (x_k / *k as f64) + (((*k - 1) / *k) as f64) * *rms;
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
    fn moving_average(&self, window: Duration, target: Option<TargetItem>) -> Self {
        let mut s = self.clone();
        s.moving_average_mut(window, target);
        s
    }
    fn moving_average_mut(&mut self, window: Duration, target: Option<TargetItem>) {
        if let Some(item) = target {
            let mask = MaskFilter {
                item,
                operand: MaskOperand::Equals,
            };
            self.mask_mut(mask);
        }
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
                TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e == epoch),
                TargetItem::EpochFlagItem(flag) => self.retain(|(_, f), _| *f == flag),
                TargetItem::ClockItem => {
                    self.retain(|_, (clk, _)| clk.is_some());
                },
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
                TargetItem::SnrItem(filter) => {
                    let filter = Snr::from(filter);
                    self.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|_, data| {
                                if let Some(snr) = data.snr {
                                    snr == filter
                                } else {
                                    false // no snr: drop out
                                }
                            });
                            obs.len() > 0
                        });
                        svs.len() > 0
                    });
                },
                TargetItem::SnrRangeItem((min, max)) => {
                    let (min, max) = (Snr::from(min), Snr::from(max));
                    self.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|_, data| {
                                if let Some(snr) = data.snr {
                                    snr >= min && snr <= max
                                } else {
                                    false // no snr: drop out
                                }
                            });
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
                TargetItem::ClockItem => {
                    self.retain(|_, (clk, _)| clk.is_none());
                },
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
                TargetItem::SnrItem(filter) => {
                    let filter = Snr::from(filter);
                    self.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|_, data| {
                                if let Some(snr) = data.snr {
                                    snr >= filter
                                } else {
                                    false // no snr: drop out
                                }
                            });
                            obs.len() > 0
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
                TargetItem::SnrItem(filter) => {
                    let filter = Snr::from(filter);
                    self.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|_, data| {
                                if let Some(snr) = data.snr {
                                    snr > filter
                                } else {
                                    false // no snr: drop out
                                }
                            });
                            obs.len() > 0
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
                TargetItem::SnrItem(filter) => {
                    let filter = Snr::from(filter);
                    self.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|_, data| {
                                if let Some(snr) = data.snr {
                                    snr <= filter
                                } else {
                                    false // no snr: drop out
                                }
                            });
                            obs.len() > 0
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
                TargetItem::SnrItem(filter) => {
                    let filter = Snr::from(filter);
                    self.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|_, data| {
                                if let Some(snr) = data.snr {
                                    snr < filter
                                } else {
                                    false // no snr: drop out
                                }
                            });
                            obs.len() > 0
                        });
                        svs.len() > 0
                    });
                },
                _ => {},
            },
        }
    }
}

impl Interpolate for Record {
    fn interpolate(&self, series: TimeSeries, target: Option<TargetItem>) -> Self {
        let mut s = self.clone();
        s.interpolate_mut(series, target);
        s
    }
    fn interpolate_mut(&mut self, series: TimeSeries, target: Option<TargetItem>) {
        if let Some(target) = target {
            let mask = MaskFilter {
                operand: MaskOperand::Equals,
                item: target,
            };
            self.mask_mut(mask);
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
            Filter::Mask(mask) => self.mask_mut(mask),
            Filter::Smoothing(filter) => match filter.stype {
                SmoothingType::Hatch => self.hatch_smoothing_mut(filter.target),
                SmoothingType::MovingAverage(dt) => self.moving_average_mut(dt, filter.target),
            },
            Filter::Interp(filter) => self.interpolate_mut(filter.series, filter.target),
            Filter::Decimation(filter) => match filter.dtype {
                DecimationType::DecimByRatio(r) => self.decimate_by_ratio_mut(r),
                DecimationType::DecimByInterval(dt) => self.decimate_by_interval_mut(dt),
            },
        }
    }
}

impl Processing for Record {
    fn min(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) {
        let mut ret: (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) = (None, HashMap::new());
        for (_, (clk, svs)) in self {
            /*
             * min{clock}
             */
            if let Some(clk) = clk {
                if let Some(data) = ret.0 {
                    if *clk < data {
                        ret.0 = Some(*clk);
                    }
                } else {
                    ret.0 = Some(*clk);
                }
            }
            /*
             * min{data}
             */
            for (sv, observables) in svs {
                for (observable, observation) in observables {
                    if let Some(data) = ret.1.get_mut(sv) {
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
                        ret.1.insert(*sv, map);
                    }
                }
            }
        }
        ret
    }
    fn min_observable(&self) -> HashMap<Observable, f64> {
        let mut ret: HashMap<Observable, f64> = HashMap::new();
        let min = self.min().1;
        for (_, observables) in min {
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
    fn max(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) {
        let mut ret: (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) = (None, HashMap::new());
        for (_, (clk, svs)) in self {
            /*
             * max{clock}
             */
            if let Some(clk) = clk {
                if let Some(mut data) = ret.0 {
                    if *clk > data {
                        data = *clk;
                    }
                } else {
                    ret.0 = Some(*clk);
                }
            }
            /*
             * max{data}
             */
            for (sv, observables) in svs {
                for (observable, observation) in observables {
                    if let Some(data) = ret.1.get_mut(sv) {
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
                        ret.1.insert(*sv, map);
                    }
                }
            }
        }
        ret
    }
    fn max_observable(&self) -> HashMap<Observable, f64> {
        let mut ret: HashMap<Observable, f64> = HashMap::new();
        let max = self.max().1;
        for (_, observables) in max {
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
    fn mean(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) {
        let mut sum: (
            Option<(u32, f64)>,
            HashMap<Sv, HashMap<Observable, (u32, f64)>>,
        ) = (None, HashMap::new());
        for (_, (clk, svs)) in self {
            /*
             * mean{clk}
             */
            if let Some(clk) = clk {
                if let Some((count, data)) = sum.0 {
                    sum.0 = Some((count + 1, data + clk));
                } else {
                    sum.0 = Some((1, *clk));
                }
            }
            /*
             * mean{data}
             */
            for (sv, observables) in svs {
                for (observable, observation) in observables {
                    if let Some(data) = sum.1.get_mut(sv) {
                        if let Some((count, sum)) = data.get_mut(observable) {
                            *count += 1;
                            *sum += observation.obs;
                        } else {
                            data.insert(observable.clone(), (1, observation.obs));
                        }
                    } else {
                        let mut map: HashMap<Observable, (u32, f64)> = HashMap::new();
                        map.insert(observable.clone(), (1, observation.obs));
                        sum.1.insert(*sv, map);
                    }
                }
            }
        }
        let mut ret: (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) = (None, HashMap::new());
        /*
         * mean{clk}
         */
        if let Some((count, data)) = sum.0 {
            ret.0 = Some(data / count as f64);
        }
        /*
         * mean{data}
         */
        for (sv, observables) in sum.1 {
            for (observable, (count, sum)) in observables {
                if let Some(data) = ret.1.get_mut(&sv) {
                    if let Some(data) = data.get_mut(&observable) {
                        *data = sum / count as f64;
                    } else {
                        data.insert(observable.clone(), sum / count as f64);
                    }
                } else {
                    let mut map: HashMap<Observable, f64> = HashMap::new();
                    map.insert(observable.clone(), sum / count as f64);
                    ret.1.insert(sv, map);
                }
            }
        }
        ret
    }
    fn mean_observable(&self) -> HashMap<Observable, f64> {
        let mean = self.mean().1;
        let mut sum: HashMap<Observable, (u32, f64)> = HashMap::new();
        for (_sv, observables) in mean {
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
            .map(|(k, (count, sum))| (k.clone(), sum / *count as f64))
            .collect()
    }
    fn stddev(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) {
        let mut stdvar = self.stdvar();
        if let Some(mut data) = stdvar.0 {
            // {clk}
            data = data.sqrt();
        }
        for (_, observables) in stdvar.1.iter_mut() {
            // {data}
            for (_observable, data) in observables.iter_mut() {
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
    fn stdvar(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) {
        self.central_moment(2)
    }
    fn central_moment(&self, order: u16) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) {
        let mean = self.mean();
        let ret: (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) = (None, HashMap::new());
        let mut diff: (
            Option<(u32, f64)>,
            HashMap<Sv, HashMap<Observable, (u32, f64)>>,
        ) = (None, HashMap::new());
        for (_, (clk, svs)) in self {
            /*
             * |x_i -x|{clk}
             */
            if let Some(clk) = clk {
                if let Some(mean) = mean.0 {
                    if let Some((mut count, mut diff)) = diff.0 {
                        count += 1;
                        diff += (*clk - mean).powf(order as f64);
                    }
                }
            }
            /*
             * |x_i -x|{data}
             */
            for (sv, observables) in svs {
                for (observable, observation) in observables {
                    let mean = mean.1.get(&sv).unwrap().get(&observable).unwrap();
                    if let Some(data) = diff.1.get_mut(sv) {
                        if let Some((count, diff)) = data.get_mut(observable) {
                            *count += 1;
                            *diff += (observation.obs - mean).powf(order as f64);
                        } else {
                            data.insert(
                                observable.clone(),
                                (1, (observation.obs - mean).powf(order as f64)),
                            );
                        }
                    } else {
                        let mut map: HashMap<Observable, (u32, f64)> = HashMap::new();
                        map.insert(
                            observable.clone(),
                            (1, (observation.obs - mean).powf(order as f64)),
                        );
                        diff.1.insert(*sv, map);
                    }
                }
            }
        }
        let mut ret: (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) = (None, HashMap::new());
        /*
         * stdvar{clk}
         */
        if let Some((count, diff)) = diff.0 {
            ret.0 = Some(diff / count as f64);
        }
        /*
         * stdvar{data}
         */
        for (sv, observables) in diff.1 {
            for (observable, (count, diff)) in observables {
                if let Some(data) = ret.1.get_mut(&sv) {
                    if let Some(data) = data.get_mut(&observable) {
                        *data = diff / count as f64;
                    } else {
                        data.insert(observable.clone(), diff / count as f64);
                    }
                } else {
                    let mut map: HashMap<Observable, f64> = HashMap::new();
                    map.insert(observable.clone(), diff / count as f64);
                    ret.1.insert(sv, map);
                }
            }
        }
        ret
    }
    fn stdvar_observable(&self) -> HashMap<Observable, f64> {
        self.central_moment_observable(2)
    }
    fn central_moment_observable(&self, order: u16) -> HashMap<Observable, f64> {
        let mean = self.mean_observable();
        let mut diff: HashMap<Observable, (u32, f64)> = HashMap::new();
        for (_, (_, svs)) in self {
            for (sv, observables) in svs {
                for (observable, observation) in observables {
                    let mean = mean.get(&observable).unwrap();
                    if let Some((count, diff)) = diff.get_mut(observable) {
                        *count += 1;
                        *diff += (observation.obs - mean).powf(order as f64);
                    } else {
                        diff.insert(
                            observable.clone(),
                            (1, (observation.obs - mean).powf(order as f64)),
                        );
                    }
                }
            }
        }
        diff.iter()
            .map(|(observable, (count, diff))| (observable.clone(), diff / *count as f64))
            .collect()
    }
    fn derivative(&self) -> Record {
        let mut prev: (
            Option<(Epoch, f64)>,
            BTreeMap<Sv, HashMap<Observable, (Epoch, f64)>>,
        ) = (None, BTreeMap::new());
        let mut ret: Record = Record::new();
        for ((epoch, flag), (clk, svs)) in self {
            /*
             * d/dt{clk}
             */
            let mut new_clk: Option<f64> = None;
            if let Some(clk) = clk {
                if let Some((mut prev_epoch, mut prev_data)) = prev.0 {
                    new_clk = Some(
                        (*clk - prev_data) / (*epoch - prev_epoch).to_unit(hifitime::Unit::Second),
                    );
                    prev_epoch = *epoch;
                    prev_data = *clk;
                } else {
                    prev.0 = Some((*epoch, *clk));
                }
            }
            /*
             * d/dt{data}
             */
            for (sv, observables) in svs {
                for (observable, observation) in observables {
                    if let Some(prev) = prev.1.get_mut(&sv) {
                        if let Some((mut prev_epoch, mut prev_data)) = prev.get_mut(&observable) {
                            if let Some((clk, data)) = ret.get_mut(&(*epoch, *flag)) {
                                *clk = new_clk;
                                if let Some(data) = data.get_mut(&sv) {
                                    if let Some(data) = data.get_mut(&observable) {
                                        *data = ObservationData {
                                            obs: (observation.obs - prev_data)
                                                / (*epoch - prev_epoch)
                                                    .to_unit(hifitime::Unit::Second),
                                            lli: data.lli,
                                            snr: data.snr,
                                        };
                                    } else {
                                        data.insert(
                                            observable.clone(),
                                            ObservationData {
                                                obs: (observation.obs - prev_data)
                                                    / (*epoch - prev_epoch)
                                                        .to_unit(hifitime::Unit::Second),
                                                lli: observation.lli,
                                                snr: observation.snr,
                                            },
                                        );
                                    }
                                } else {
                                    let mut map: HashMap<Observable, ObservationData> =
                                        HashMap::new();
                                    map.insert(
                                        observable.clone(),
                                        ObservationData {
                                            obs: (observation.obs - prev_data)
                                                / (*epoch - prev_epoch)
                                                    .to_unit(hifitime::Unit::Second),
                                            lli: observation.lli,
                                            snr: observation.snr,
                                        },
                                    );
                                    data.insert(*sv, map);
                                }
                            } else {
                                let mut map: HashMap<Observable, ObservationData> = HashMap::new();
                                map.insert(
                                    observable.clone(),
                                    ObservationData {
                                        obs: (observation.obs - prev_data)
                                            / (*epoch - prev_epoch).to_unit(hifitime::Unit::Second),
                                        lli: observation.lli,
                                        snr: observation.snr,
                                    },
                                );
                                let mut mmap: BTreeMap<Sv, HashMap<Observable, ObservationData>> =
                                    BTreeMap::new();
                                mmap.insert(*sv, map);
                                ret.insert((*epoch, *flag), (new_clk, mmap));
                            }
                            prev_epoch = *epoch;
                            prev_data = observation.obs;
                        } else {
                            prev.insert(observable.clone(), (*epoch, observation.obs));
                        }
                    } else {
                        let mut map: HashMap<Observable, (Epoch, f64)> = HashMap::new();
                        map.insert(observable.clone(), (*epoch, observation.obs));
                        prev.1.insert(*sv, map);
                    }
                }
            }
        }
        ret
    }
    fn skewness(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) {
        let stddev = self.stddev();
        let central_moment = self.central_moment(3);
        let mut ret: (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) =
            (None, HashMap::with_capacity(stddev.1.len()));
        /*
         * skew{clk}
         */
        if let Some(stddev) = stddev.0 {
            if let Some(moment) = central_moment.0 {
                ret.0 = Some(moment / stddev.powf(3.0));
            }
        }
        /*
         * skew{data}
         */
        for (sv, observables) in stddev.1 {
            if let Some(moment_observables) = central_moment.1.get(&sv) {
                for (observable, stddev) in observables {
                    if let Some(moment) = moment_observables.get(&observable) {
                        if let Some(data) = ret.1.get_mut(&sv) {
                            data.insert(observable.clone(), moment / stddev.powf(3.0));
                        } else {
                            let mut map: HashMap<Observable, f64> = HashMap::new();
                            map.insert(observable.clone(), moment / stddev.powf(3.0));
                            ret.1.insert(sv, map);
                        }
                    }
                }
            }
        }
        ret
    }
    fn skewness_observable(&self) -> HashMap<Observable, f64> {
        let stddev = self.stddev_observable();
        let central_moment = self.central_moment_observable(3);
        let mut ret: HashMap<Observable, f64> = HashMap::with_capacity(stddev.len());
        for (observable, stddev) in stddev {
            if let Some(moment) = central_moment.get(&observable) {
                ret.insert(observable.clone(), moment / stddev.powf(3.0));
            }
        }
        ret
    }
}

/*
 * Forms all GF combinations
 */
fn gf_combination(
    record: &Record,
) -> HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
    let mut ret: HashMap<
        (Observable, Observable),
        BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>,
    > = HashMap::new();
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
                            let carrier = ref_observable.carrier(sv.constellation).unwrap();
                            reference = Some((
                                ref_observable.clone(),
                                (
                                    carrier.carrier_wavelength(),
                                    carrier.carrier_frequency(),
                                    ref_data.obs,
                                ),
                            ));
                        } else {
                            reference = Some((ref_observable.clone(), (1.0, 1.0, ref_data.obs)));
                        }
                        break; // DONE searching
                    }
                }

                if let Some((ref_observable, (ref_lambda, ref_freq, ref_data))) = reference {
                    // got a reference
                    let gf = match ref_observable.is_phase_observable() {
                        true => {
                            let carrier = lhs_observable.carrier(sv.constellation).unwrap();
                            let lhs_lambda = carrier.carrier_wavelength();
                            let lhs_freq = carrier.carrier_frequency();
                            let gamma = lhs_freq / ref_freq;
                            let total_scaling = 1.0 / (gamma.powf(2.0) - 1.0);
                            (lhs_data.obs * lhs_lambda - ref_data * ref_lambda) * total_scaling
                        },
                        false => ref_data - lhs_data.obs, // PR: sign differs
                    };

                    if let Some(data) =
                        ret.get_mut(&(lhs_observable.clone(), ref_observable.clone()))
                    {
                        if let Some(data) = data.get_mut(&sv) {
                            data.insert(*epoch, gf);
                        } else {
                            let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                            bmap.insert(*epoch, gf);
                            data.insert(*sv, bmap);
                        }
                    } else {
                        // new combination
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
                            let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                            bmap.insert(*epoch, gf);
                            let mut map: BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>> =
                                BTreeMap::new();
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
fn nl_combination(
    record: &Record,
) -> HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
    let mut ret: HashMap<
        (Observable, Observable),
        BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>,
    > = HashMap::new();
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

                if let Some((ref_observable, ref_data)) = reference {
                    // got a reference
                    let gf = match ref_observable.is_phase_observable() {
                        true => lhs_data.obs - ref_data,
                        false => ref_data - lhs_data.obs, // PR: sign differs
                    };

                    if let Some(data) =
                        ret.get_mut(&(lhs_observable.clone(), ref_observable.clone()))
                    {
                        if let Some(data) = data.get_mut(&sv) {
                            data.insert(*epoch, gf);
                        } else {
                            let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                            bmap.insert(*epoch, gf);
                            data.insert(*sv, bmap);
                        }
                    } else {
                        // new combination
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
                            let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                            bmap.insert(*epoch, gf);
                            let mut map: BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>> =
                                BTreeMap::new();
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
fn wl_combination(
    record: &Record,
) -> HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
    let mut ret: HashMap<
        (Observable, Observable),
        BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>,
    > = HashMap::new();
    for (epoch, (_, vehicules)) in record {
        for (sv, observations) in vehicules {
            for (lhs_observable, lhs_data) in observations {
                if !lhs_observable.is_phase_observable() {
                    continue; // only on phase data
                }
                let lhs_code = lhs_observable.to_string();
                let lhs_carrier = &lhs_code[1..2];

                // determine another carrier
                let rhs_carrier = match lhs_carrier {
                    // this will restrict combinations to
                    "1" => "2", // 1 against 2
                    _ => "1",   // M > 1 against 1
                };

                let lhs_carrier =
                    Carrier::from_observable(sv.constellation, lhs_observable).unwrap();

                // locate a reference code against another carrier
                let mut reference: Option<(Observable, f64, f64)> = None;
                for (ref_observable, ref_data) in observations {
                    if ref_observable == lhs_observable {
                        continue; // must differ
                    }
                    let both_phase = ref_observable.is_phase_observable()
                        && lhs_observable.is_phase_observable();
                    if !both_phase {
                        continue;
                    }

                    let refcode = ref_observable.to_string();
                    let carrier_code = &refcode[1..2];
                    if carrier_code != rhs_carrier {
                        let rhs_carrier =
                            Carrier::from_observable(sv.constellation, ref_observable).unwrap();
                        reference = Some((
                            ref_observable.clone(),
                            ref_data.obs,
                            rhs_carrier.carrier_frequency(),
                        ));
                        break; // DONE searching
                    }
                }

                if let Some((ref_observable, ref_data, ref_freq)) = reference {
                    // got a reference
                    let yp = 299_792_458.0_f64 * (lhs_data.obs - ref_data)
                        / (lhs_carrier.carrier_frequency() - ref_freq);

                    if let Some(data) =
                        ret.get_mut(&(lhs_observable.clone(), ref_observable.clone()))
                    {
                        if let Some(data) = data.get_mut(&sv) {
                            data.insert(*epoch, yp);
                        } else {
                            let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                            bmap.insert(*epoch, yp);
                            data.insert(*sv, bmap);
                        }
                    } else {
                        // new combination
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
                            let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                            bmap.insert(*epoch, yp);
                            let mut map: BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>> =
                                BTreeMap::new();
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
fn mw_combination(
    record: &Record,
) -> HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
    let mut ret: HashMap<
        (Observable, Observable),
        BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>,
    > = HashMap::new();
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

                if let Some((ref_observable, ref_data)) = reference {
                    // got a reference
                    let gf = match ref_observable.is_phase_observable() {
                        true => lhs_data.obs - ref_data,
                        false => ref_data - lhs_data.obs, // PR: sign differs
                    };

                    if let Some(data) =
                        ret.get_mut(&(lhs_observable.clone(), ref_observable.clone()))
                    {
                        if let Some(data) = data.get_mut(&sv) {
                            data.insert(*epoch, gf);
                        } else {
                            let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                            bmap.insert(*epoch, gf);
                            data.insert(*sv, bmap);
                        }
                    } else {
                        // new combination
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
                            let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                            bmap.insert(*epoch, gf);
                            let mut map: BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>> =
                                BTreeMap::new();
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
    fn combine(
        &self,
        combination: Combination,
    ) -> HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
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

                    for rhs_code in carrier::KNOWN_CODES.iter() {
                        // locate a reference code
                        if *rhs_code != lhs_code {
                            // code differs
                            if rhs_code.starts_with(lhs_carrier) {
                                // same carrier
                                let tolocate = match lhs_observable.is_phase_observable() {
                                    true => "L".to_owned() + rhs_code,  // same physics
                                    false => "C".to_owned() + rhs_code, // same physics
                                };
                                let tolocate = Observable::from_str(&tolocate).unwrap();
                                if let Some(rhs_observation) = observations.get(&tolocate) {
                                    // got a reference code
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
                                            } else {
                                                // code is refered to
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
                                        bmap.insert(
                                            *epoch,
                                            lhs_observation.obs - rhs_observation.obs,
                                        );
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
    fn iono_delay_detector(
        &self,
        max_dt: Duration,
    ) -> HashMap<Observable, HashMap<Sv, BTreeMap<Epoch, f64>>> {
        let gf = self.combine(Combination::GeometryFree);
        let mut ret: HashMap<Observable, HashMap<Sv, BTreeMap<Epoch, f64>>> = HashMap::new();
        let mut prev_data: HashMap<(Observable, Observable), HashMap<Sv, (Epoch, f64)>> =
            HashMap::new();
        for (combination, vehicles) in gf {
            let (_lhs_observable, ref_observable) = combination.clone();
            if !ref_observable.is_phase_observable() {
                continue; // only on phase data
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
                                    let mut map: HashMap<Sv, BTreeMap<Epoch, f64>> = HashMap::new();
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
    fn obs_record_is_new_epoch() {
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
