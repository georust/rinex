use bitflags::bitflags;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use thiserror::Error;

use crate::{
    epoch, merge, merge::Merge, prelude::*, split, split::Split, types::Type, version::Version,
    Carrier, Observable,
};

use crate::observation::SNR;
use hifitime::Duration;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse epoch")]
    EpochError(#[from] epoch::ParsingError),
    #[error("constellation parsing error")]
    ConstellationParsing(#[from] gnss::constellation::ParsingError),
    #[error("sv parsing error")]
    SvParsing(#[from] gnss::sv::ParsingError),
    #[error("failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse vehicles properly (nb_sat mismatch)")]
    EpochParsingError,
    #[error("line is empty")]
    MissingData,
}

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    #[derive(Debug, Copy, Clone)]
    #[derive(PartialEq, PartialOrd)]
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

#[derive(Default, Copy, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ObservationData {
    /// physical measurement
    pub obs: f64,
    /// Lock loss indicator
    pub lli: Option<LliFlags>,
    /// Signal strength indicator
    pub snr: Option<SNR>,
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
    pub fn new(obs: f64, lli: Option<LliFlags>, snr: Option<SNR>) -> ObservationData {
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
        let snr_ok = self.snr.unwrap_or(SNR::default()).strong();
        lli_ok && snr_ok
    }

    /// Returns true if self is considered Ok with respect to given
    /// SNR condition (>=)
    pub fn is_ok_snr(&self, min_snr: SNR) -> bool {
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

/// Observation Record content, sorted by [`Epoch`], per [`SV`] and per
/// [`Observable`].
pub type Record = BTreeMap<
    (Epoch, EpochFlag),
    (
        Option<f64>,
        BTreeMap<SV, HashMap<Observable, ObservationData>>,
    ),
>;

/// Returns true if given content matches a new OBSERVATION data epoch
pub(crate) fn is_new_epoch(line: &str, v: Version) -> bool {
    if v.major < 3 {
        if line.len() < 30 {
            false
        } else {
            epoch::parse_utc(&line[0..29]).is_ok()
        }
    } else {
        // Modern RINEX
        // OBS::V3 behaves like all::V4
        match line.chars().next() {
            Some(c) => {
                c == '>' // epochs always delimited
                         // by this new identifier
            },
            _ => false,
        }
    }
}

/// Builds `Record` entry for `ObservationData` from given epoch content
pub(crate) fn parse_epoch(
    header: &Header,
    content: &str,
    ts: TimeScale,
) -> Result<
    (
        (Epoch, EpochFlag),
        Option<f64>,
        BTreeMap<SV, HashMap<Observable, ObservationData>>,
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
    if line.starts_with('>') {
        line = line.split_at(1).1;
    }

    let (date, rem) = line.split_at(offset + 3);
    let (n_sat, rem) = rem.split_at(3);
    let n_sat = n_sat.trim().parse::<u16>()?;
    let epoch = epoch::parse_in_timescale(date, ts)?;

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
            parse_v2(header, &systems, observables, lines)
        },
        _ => parse_v3(observables, lines),
    };
    Ok((epoch, clock_offset, data))
}

/*
 * Parses a V2 epoch from given lines iteratoor
 * Vehicle description is contained in the epoch descriptor
 * Each vehicle content is wrapped into several lines
 */
fn parse_v2(
    header: &Header,
    systems: &str,
    header_observables: &HashMap<Constellation, Vec<Observable>>,
    lines: std::str::Lines<'_>,
) -> BTreeMap<SV, HashMap<Observable, ObservationData>> {
    let svnn_size = 3; // SVNN standard
    let nb_max_observables = 5; // in a single line
    let observable_width = 16; // data + 2 flags + 1 whitespace
    let mut sv_ptr = 0; // svnn pointer
    let mut obs_ptr = 0; // observable pointer
    let mut data: BTreeMap<SV, HashMap<Observable, ObservationData>> = BTreeMap::new();
    let mut inner: HashMap<Observable, ObservationData> = HashMap::with_capacity(5);
    let mut sv = SV::default();
    let mut observables: &Vec<Observable>;
    //println!("{:?}", header_observables); // DEBUG
    //println!("\"{}\"", systems); // DEBUG

    // parse first system we're dealing with
    if systems.len() < svnn_size {
        // Can't even parse a single vehicle;
        // epoch descriptor is totally corrupt, stop here
        return data;
    }

    /*
     * identify 1st system
     */
    let max = std::cmp::min(svnn_size, systems.len()); // for epochs with a single vehicle
    let system = &systems[0..max];

    if let Ok(ssv) = SV::from_str(system) {
        sv = ssv;
    } else {
        // may fail on omitted X in "XYY",
        // mainly on OLD RINEX with mono constellation
        match header.constellation {
            Some(Constellation::Mixed) => panic!("bad gnss definition"),
            Some(c) => {
                if let Ok(prn) = system.trim().parse::<u8>() {
                    if let Ok(s) = SV::from_str(&format!("{}{:02}", c, prn)) {
                        sv = s;
                    } else {
                        return data;
                    }
                }
            },
            None => return data,
        }
    }
    sv_ptr += svnn_size; // increment pointer
                         //println!("\"{}\"={}", system, sv); // DEBUG

    // grab observables for this vehicle
    observables = match sv.constellation.is_sbas() {
        true => {
            if let Some(observables) = header_observables.get(&Constellation::SBAS) {
                observables
            } else {
                // failed to identify observations for this vehicle
                return data;
            }
        },
        false => {
            if let Some(observables) = header_observables.get(&sv.constellation) {
                observables
            } else {
                // failed to identify observations for this vehicle
                return data;
            }
        },
    };
    //println!("{:?}", observables); // DEBUG

    for line in lines {
        // browse all lines provided
        //println!("parse_v2: \"{}\"", line); //DEBUG
        let line_width = line.len();
        if line_width < 10 {
            //println!("\nEMPTY LINE: \"{}\"", line); //DEBUG
            // line is empty
            // add maximal amount of vehicles possible
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
                let mut snr: Option<SNR> = None;
                if let Ok(obs) = obs.trim().parse::<f64>() {
                    // parse obs
                    if slice.len() > 14 {
                        let lli_str = &slice[14..15];
                        if let Ok(u) = lli_str.parse::<u8>() {
                            lli = LliFlags::from_bits(u);
                        }
                        if slice.len() > 15 {
                            let snr_str = &slice[15..16];
                            if let Ok(s) = SNR::from_str(snr_str) {
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
            // we're done with current vehicle
            // build data
            data.insert(sv, inner.clone());
            inner.clear(); // prepare for next vehicle
            obs_ptr = 0;
            //identify next vehicle
            if sv_ptr >= systems.len() {
                // last vehicle
                return data;
            }
            // identify next vehicle
            let start = sv_ptr;
            let end = std::cmp::min(sv_ptr + svnn_size, systems.len()); // trimed epoch description
            let system = &systems[start..end];
            if let Ok(s) = SV::from_str(system) {
                sv = s;
            } else {
                // may fail on omitted X in "XYY",
                // mainly on OLD RINEX with mono constellation
                match header.constellation {
                    Some(c) => {
                        if let Ok(prn) = system.trim().parse::<u8>() {
                            if let Ok(s) = SV::from_str(&format!("{}{:02}", c, prn)) {
                                sv = s;
                            } else {
                                return data;
                            }
                        }
                    },
                    _ => unreachable!(),
                }
            }
            //println!("\"{}\"={}", system, sv); //DEBUG
            sv_ptr += svnn_size; // increment pointer

            // grab observables for this vehicle
            observables = match sv.constellation.is_sbas() {
                true => {
                    if let Some(observables) = header_observables.get(&Constellation::SBAS) {
                        observables
                    } else {
                        // failed to identify observations for this vehicle
                        return data;
                    }
                },
                false => {
                    if let Some(observables) = header_observables.get(&sv.constellation) {
                        observables
                    } else {
                        // failed to identify observations for this vehicle
                        return data;
                    }
                },
            };
            //println!("{:?}", observables); // DEBUG
        }
    } // for all lines provided
    data
}

/*
 * Parses a V3 epoch from given lines iteratoor
 * Format is much simpler, one vehicle is described in a single line
 */
fn parse_v3(
    observables: &HashMap<Constellation, Vec<Observable>>,
    lines: std::str::Lines<'_>,
) -> BTreeMap<SV, HashMap<Observable, ObservationData>> {
    let svnn_size = 3; // SVNN standard
    let observable_width = 16; // data + 2 flags
    let mut data: BTreeMap<SV, HashMap<Observable, ObservationData>> = BTreeMap::new();
    let mut inner: HashMap<Observable, ObservationData> = HashMap::with_capacity(5);
    for line in lines {
        // browse all lines
        //println!("parse_v3: \"{}\"", line); //DEBUG
        let (sv, line) = line.split_at(svnn_size);
        if let Ok(sv) = SV::from_str(sv) {
            let obscodes = match sv.constellation.is_sbas() {
                true => observables.get(&Constellation::SBAS),
                false => observables.get(&sv.constellation),
            };
            //println!("SV: {} OBSERVABLES: {:?}", sv, obscodes); // DEBUG
            if let Some(obscodes) = obscodes {
                let nb_obs = line.len() / observable_width;
                inner.clear();
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
                    rem = r;
                    let content_len = content.len();
                    let mut snr: Option<SNR> = None;
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
                            if let Ok(s) = SNR::from_str(snr_str) {
                                snr = Some(s);
                            }
                        }
                        //println!("LLI {:?}", lli); //DEBUG
                        //println!("SSI {:?}", snr);
                        // build content
                        inner.insert(obscodes[i].clone(), ObservationData { obs, lli, snr });
                    }
                }
                if rem.len() >= observable_width - 2 {
                    let mut snr: Option<SNR> = None;
                    let mut lli: Option<LliFlags> = None;
                    let obs = &rem[0..observable_width - 2];
                    if let Ok(obs) = obs.trim().parse::<f64>() {
                        if rem.len() > observable_width - 2 {
                            let lli_str = &rem[observable_width - 2..observable_width - 1];
                            if let Ok(u) = lli_str.parse::<u8>() {
                                lli = LliFlags::from_bits(u);
                                if rem.len() > observable_width - 1 {
                                    let snr_str = &rem[observable_width - 1..];
                                    if let Ok(s) = SNR::from_str(snr_str) {
                                        snr = Some(s);
                                    }
                                }
                            }
                        }
                        inner.insert(obscodes[nb_obs].clone(), ObservationData { obs, lli, snr });
                    }
                }
                if !inner.is_empty() {
                    data.insert(sv, inner.clone());
                }
            } //got some observables to work with
        } // SV::from_str failed()
    } //browse all lines
    data
}

/// Formats one epoch according to standard definitions
pub(crate) fn fmt_epoch(
    epoch: Epoch,
    flag: EpochFlag,
    clock_offset: &Option<f64>,
    data: &BTreeMap<SV, HashMap<Observable, ObservationData>>,
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
    data: &BTreeMap<SV, HashMap<Observable, ObservationData>>,
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

    lines.push('\n');
    for (sv, data) in data.iter() {
        lines.push_str(&format!("{:x}", sv));
        let observables = match sv.constellation.is_sbas() {
            true => observables.get(&Constellation::SBAS),
            false => observables.get(&sv.constellation),
        };
        if let Some(observables) = observables {
            for observable in observables {
                if let Some(observation) = data.get(observable) {
                    lines.push_str(&format!("{:14.3}", observation.obs));
                    if let Some(flag) = observation.lli {
                        lines.push_str(&format!("{}", flag.bits()));
                    } else {
                        lines.push(' ');
                    }
                    if let Some(flag) = observation.snr {
                        lines.push_str(&format!("{:x}", flag));
                    } else {
                        lines.push(' ');
                    }
                } else {
                    lines.push_str("                ");
                }
            }
        }
        lines.push('\n');
    }
    lines
}

fn fmt_epoch_v2(
    epoch: Epoch,
    flag: EpochFlag,
    clock_offset: &Option<f64>,
    data: &BTreeMap<SV, HashMap<Observable, ObservationData>>,
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
            lines.push_str("\n                                ");
        }
        lines.push_str(&format!("{:x}", sv));
        index += 1;
    }
    let obs_per_line = 5;
    // for each vehicle per epoch
    for (sv, observations) in data.iter() {
        // follow list of observables, as described in header section
        // for given constellation
        let observables = match sv.constellation.is_sbas() {
            true => observables.get(&Constellation::SBAS),
            false => observables.get(&sv.constellation),
        };
        if let Some(observables) = observables {
            for (obs_index, observable) in observables.iter().enumerate() {
                if obs_index % obs_per_line == 0 {
                    lines.push('\n');
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
    lines.push('\n');
    lines
}

impl Merge for Record {
    /// Merge `rhs` into `Self`
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merge `rhs` into `Self`
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        for (rhs_epoch, (rhs_clk, rhs_vehicles)) in rhs {
            if let Some((clk, vehicles)) = self.get_mut(rhs_epoch) {
                // exact epoch (both timestamp and flag) did exist
                //  --> overwrite clock field (as is)
                *clk = *rhs_clk;
                // other fields:
                // either insert (if did not exist), or overwrite
                for (rhs_vehicle, rhs_observations) in rhs_vehicles {
                    if let Some(observations) = vehicles.get_mut(rhs_vehicle) {
                        for (rhs_observable, rhs_data) in rhs_observations {
                            if let Some(data) = observations.get_mut(rhs_observable) {
                                *data = *rhs_data; // overwrite
                            } else {
                                // new observation: insert it
                                observations.insert(rhs_observable.clone(), *rhs_data);
                            }
                        }
                    } else {
                        // new SV: insert it
                        vehicles.insert(*rhs_vehicle, rhs_observations.clone());
                    }
                }
            } else {
                // this epoch did not exist previously: insert it
                self.insert(*rhs_epoch, (*rhs_clk, rhs_vehicles.clone()));
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
                    Some((*k, v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self
            .iter()
            .flat_map(|(k, v)| {
                if k.0 >= epoch {
                    Some((*k, v.clone()))
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

#[cfg(feature = "processing")]
use crate::preprocessing::*;

#[cfg(feature = "processing")]
impl Smooth for Record {
    fn hatch_smoothing(&self) -> Self {
        let mut s = self.clone();
        s.hatch_smoothing_mut();
        s
    }
    fn hatch_smoothing_mut(&mut self) {
        // buffer:
        // stores n index, previously associated phase point and previous result
        // for every observable we're computing
        let mut buffer: HashMap<SV, HashMap<Observable, (f64, f64, f64)>> = HashMap::new();
        // for each pseudo range observation for all epochs,
        // the operation is only feasible if an associated phase_point exists
        //   Ex: C1C with L1C, not L1W
        //   and C2P with L2P not L2W
        for (_, (_, svs)) in self.iter_mut() {
            for (sv, observables) in svs.iter_mut() {
                let rhs_observables = observables.clone();
                for (pr_observable, pr_observation) in observables.iter_mut() {
                    if !pr_observable.is_pseudorange_observable() {
                        continue;
                    }

                    let pr_code = pr_observable.code().unwrap();

                    // locate associated L code
                    let ph_tolocate = "L".to_owned() + &pr_code;

                    let mut ph_data: Option<f64> = None;
                    for (rhs_observable, rhs_observation) in &rhs_observables {
                        let rhs_code = rhs_observable.to_string();
                        if rhs_code == ph_tolocate {
                            ph_data = Some(rhs_observation.obs);
                            break;
                        }
                    }

                    if ph_data.is_none() {
                        continue; // can't progress at this point
                    }

                    let phase_data = ph_data.unwrap();

                    if let Some(data) = buffer.get_mut(sv) {
                        if let Some((n, prev_result, prev_phase)) = data.get_mut(pr_observable) {
                            let delta_phase = phase_data - *prev_phase;
                            // implement corrector equation
                            pr_observation.obs = 1.0 / *n * pr_observation.obs
                                + (*n - 1.0) / *n * (*prev_result + delta_phase);
                            // update buffer storage for next iteration
                            *n += 1.0_f64;
                            *prev_result = pr_observation.obs;
                            *prev_phase = phase_data;
                        } else {
                            // first time we encounter this observable
                            // initiate buffer
                            data.insert(
                                pr_observable.clone(),
                                (2.0_f64, pr_observation.obs, phase_data),
                            );
                        }
                    } else {
                        // first time we encounter this sv
                        // pr observation is untouched on S(0)
                        // initiate buffer
                        let mut map: HashMap<Observable, (f64, f64, f64)> = HashMap::new();
                        map.insert(
                            pr_observable.clone(),
                            (2.0_f64, pr_observation.obs, phase_data),
                        );
                        buffer.insert(*sv, map);
                    }
                }
            }
        }
    }
    fn moving_average(&self, window: Duration) -> Self {
        let mut s = self.clone();
        s.moving_average_mut(window);
        s
    }
    fn moving_average_mut(&mut self, _window: Duration) {
        unimplemented!("observation:record:mov_average_mut()")
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
            MaskOperand::Equals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e == epoch),
                TargetItem::EpochFlagItem(flag) => self.retain(|(_, f), _| *f == flag),
                TargetItem::ClockItem => {
                    self.retain(|_, (clk, _)| clk.is_some());
                },
                TargetItem::ConstellationItem(constells) => {
                    let mut broad_sbas_filter = false;
                    for c in &constells {
                        broad_sbas_filter |= *c == Constellation::SBAS;
                    }
                    self.retain(|_, (_, svs)| {
                        svs.retain(|sv, _| {
                            if broad_sbas_filter {
                                sv.constellation.is_sbas() || constells.contains(&sv.constellation)
                            } else {
                                constells.contains(&sv.constellation)
                            }
                        });
                        !svs.is_empty()
                    });
                },
                TargetItem::SvItem(items) => {
                    self.retain(|_, (_, svs)| {
                        svs.retain(|sv, _| items.contains(sv));
                        !svs.is_empty()
                    });
                },
                TargetItem::ObservableItem(filter) => {
                    self.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|code, _| filter.contains(code));
                            !obs.is_empty()
                        });
                        !svs.is_empty()
                    });
                },
                TargetItem::SNRItem(filter) => {
                    let filter = SNR::from(filter);
                    self.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|_, data| {
                                if let Some(snr) = data.snr {
                                    snr == filter
                                } else {
                                    false // no snr: drop out
                                }
                            });
                            !obs.is_empty()
                        });
                        !svs.is_empty()
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
                        !svs.is_empty()
                    });
                },
                TargetItem::SvItem(items) => {
                    self.retain(|_, (_, svs)| {
                        svs.retain(|sv, _| !items.contains(sv));
                        !svs.is_empty()
                    });
                },
                TargetItem::ObservableItem(filter) => {
                    self.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|code, _| !filter.contains(code));
                            !obs.is_empty()
                        });
                        !svs.is_empty()
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
                        !svs.is_empty()
                    });
                },
                TargetItem::SNRItem(filter) => {
                    let filter = SNR::from(filter);
                    self.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|_, data| {
                                if let Some(snr) = data.snr {
                                    snr >= filter
                                } else {
                                    false // no snr: drop out
                                }
                            });
                            !obs.is_empty()
                        });
                        !svs.is_empty()
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
                        !svs.is_empty()
                    });
                },
                TargetItem::SNRItem(filter) => {
                    let filter = SNR::from(filter);
                    self.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|_, data| {
                                if let Some(snr) = data.snr {
                                    snr > filter
                                } else {
                                    false // no snr: drop out
                                }
                            });
                            !obs.is_empty()
                        });
                        !svs.is_empty()
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
                        !svs.is_empty()
                    });
                },
                TargetItem::SNRItem(filter) => {
                    let filter = SNR::from(filter);
                    self.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|_, data| {
                                if let Some(snr) = data.snr {
                                    snr <= filter
                                } else {
                                    false // no snr: drop out
                                }
                            });
                            !obs.is_empty()
                        });
                        !svs.is_empty()
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
                        !svs.is_empty()
                    });
                },
                TargetItem::SNRItem(filter) => {
                    let filter = SNR::from(filter);
                    self.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|_, data| {
                                if let Some(snr) = data.snr {
                                    snr < filter
                                } else {
                                    false // no snr: drop out
                                }
                            });
                            !obs.is_empty()
                        });
                        !svs.is_empty()
                    });
                },
                _ => {},
            },
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
        unimplemented!("observation:record:interpolate_mut()")
    }
}

/*
 * Decimates only a given record subset
 */
#[cfg(feature = "processing")]
fn decimate_data_subset(record: &mut Record, subset: &Record, target: &TargetItem) {
    match target {
        TargetItem::ClockItem => {
            /*
             * Remove clock fields from self
             * where it should now be missing
             */
            for (epoch, (clk, _)) in record.iter_mut() {
                if subset.get(epoch).is_none() {
                    // should be missing
                    *clk = None; // now missing
                }
            }
        },
        TargetItem::SvItem(svs) => {
            /*
             * Remove SV observations where it should now be missing
             */
            for (epoch, (_, vehicles)) in record.iter_mut() {
                if subset.get(epoch).is_none() {
                    // should be missing
                    for sv in svs.iter() {
                        vehicles.remove(sv); // now missing
                    }
                }
            }
        },
        TargetItem::ObservableItem(obs_list) => {
            /*
             * Remove given observations where it should now be missing
             */
            for (epoch, (_, vehicles)) in record.iter_mut() {
                if subset.get(epoch).is_none() {
                    // should be missing
                    for (_sv, observables) in vehicles.iter_mut() {
                        observables.retain(|observable, _| !obs_list.contains(observable));
                    }
                }
            }
        },
        TargetItem::ConstellationItem(constells_list) => {
            /*
             * Remove observations for given constellation(s) where it should now be missing
             */
            for (epoch, (_, vehicles)) in record.iter_mut() {
                if subset.get(epoch).is_none() {
                    // should be missing
                    vehicles.retain(|sv, _| {
                        let mut contained = false;
                        for constell in constells_list.iter() {
                            if sv.constellation == *constell {
                                contained = true;
                                break;
                            }
                        }
                        !contained
                    });
                }
            }
        },
        TargetItem::SNRItem(_) => unimplemented!("decimate_data_subset::snr"),
        _ => {},
    }
}

#[cfg(feature = "processing")]
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
                SmoothingType::Hatch => {
                    if filter.target.is_none() {
                        self.hatch_smoothing_mut();
                        return; // no need to proceed further
                    }

                    let item = filter.target.unwrap();

                    // apply mask to retain desired subset
                    let mask = MaskFilter {
                        item: item.clone(),
                        operand: MaskOperand::Equals,
                    };

                    // apply smoothing
                    let mut subset = self.mask(mask);
                    subset.hatch_smoothing_mut();
                    // overwrite targetted content
                    let _ = self.merge_mut(&subset); // cannot fail here (record types match)
                },
                SmoothingType::MovingAverage(dt) => {
                    if filter.target.is_none() {
                        self.moving_average_mut(dt);
                        return; // no need to proceed further
                    }

                    let item = filter.target.unwrap();

                    // apply mask to retain desired subset
                    let mask = MaskFilter {
                        item: item.clone(),
                        operand: MaskOperand::Equals,
                    };

                    // apply smoothing
                    let mut subset = self.mask(mask);
                    subset.moving_average_mut(dt);
                    // overwrite targetted content
                    let _ = self.merge_mut(&subset); // cannot fail here (record types match)
                },
            },
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

                    // and decimate
                    let subset = self.mask(mask).decimate_by_ratio(r);

                    // adapt self's subset to new data rates
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

                    // and decimate
                    let subset = self.mask(mask).decimate_by_interval(dt);

                    // adapt self's subset to new data rates
                    decimate_data_subset(self, &subset, &item);
                },
            },
        }
    }
}

#[cfg(feature = "processing")]
impl Decimate for Record {
    fn decimate_by_ratio_mut(&mut self, r: u32) {
        let mut i = 0;
        self.retain(|_, _| {
            let retained = (i % r) == 0;
            i += 1;
            retained
        });
    }
    fn decimate_by_ratio(&self, r: u32) -> Self {
        let mut s = self.clone();
        s.decimate_by_ratio_mut(r);
        s
    }
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
        s.decimate_match_mut(rhs);
        s
    }
}

#[cfg(feature = "obs")]
use crate::observation::{Combination, Combine};

/*
 * Combines same physics but observed on different carrier frequency
 */
#[cfg(feature = "obs")]
fn dual_freq_combination(
    rec: &Record,
    combination: Combination,
) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
    const SPEED_OF_LIGHT: f64 = 2.99792458E8;
    let mut ret: HashMap<
        (Observable, Observable),
        BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>,
    > = HashMap::new();
    for (epoch, (_, vehicles)) in rec {
        for (sv, observations) in vehicles {
            for (lhs_observable, lhs_data) in observations {
                if !lhs_observable.is_phase_observable()
                    && !lhs_observable.is_pseudorange_observable()
                {
                    continue; // only for these two physics
                }

                // consider anything but L1
                let lhs_code = lhs_observable.to_string();
                let lhs_is_l1 = lhs_code.contains("1");
                if lhs_is_l1 {
                    continue;
                }

                // find L1 reference observation
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
                    if refcode.contains("1") {
                        reference = Some((ref_observable.clone(), ref_data.obs));
                        break; // DONE searching
                    }
                }

                if reference.is_none() {
                    continue; // can't proceed further
                }
                let (ref_observable, ref_data) = reference.unwrap();

                // determine frequencies
                let lhs_carrier = Carrier::from_observable(sv.constellation, &lhs_observable);
                let ref_carrier = Carrier::from_observable(sv.constellation, &ref_observable);
                if lhs_carrier.is_err() | ref_carrier.is_err() {
                    continue; // undetermined frequency
                }

                let (lhs_carrier, ref_carrier) = (lhs_carrier.unwrap(), ref_carrier.unwrap());
                let (fj, fi) = (lhs_carrier.frequency(), ref_carrier.frequency());
                let (lambda_j, lambda_i) = (lhs_carrier.wavelength(), ref_carrier.wavelength());

                let alpha = match combination {
                    Combination::GeometryFree => 1.0_f64,
                    Combination::IonosphereFree => 1.0 / (fi.powi(2) - fj.powi(2)),
                    Combination::WideLane => 1.0 / (fi - fj),
                    Combination::NarrowLane => 1.0 / (fi + fj),
                    Combination::MelbourneWubbena => unreachable!("mw combination"),
                };

                let beta = match combination {
                    Combination::GeometryFree => 1.0_f64,
                    Combination::IonosphereFree => {
                        if ref_observable.is_pseudorange_observable() {
                            fi.powi(2)
                        } else {
                            SPEED_OF_LIGHT * fi
                        }
                    },
                    Combination::WideLane | Combination::NarrowLane => {
                        if ref_observable.is_pseudorange_observable() {
                            1.0_f64
                        } else {
                            SPEED_OF_LIGHT
                        }
                    },
                    Combination::MelbourneWubbena => unreachable!("mw combination"),
                };

                let gamma = match combination {
                    Combination::GeometryFree => 1.0_f64,
                    Combination::IonosphereFree => {
                        if ref_observable.is_pseudorange_observable() {
                            fj.powi(2) / fi.powi(2)
                        } else {
                            fj / fi
                        }
                    },
                    Combination::WideLane => 1.0_f64,
                    Combination::NarrowLane => 1.0_f64,
                    Combination::MelbourneWubbena => unreachable!("mw combination"),
                };

                let (v_j, v_i) = match combination {
                    Combination::GeometryFree => {
                        if ref_observable.is_pseudorange_observable() {
                            (ref_data, lhs_data.obs)
                        } else {
                            (lhs_data.obs * lambda_j, ref_data * lambda_i)
                        }
                    },
                    _ => {
                        if ref_observable.is_pseudorange_observable() {
                            (lhs_data.obs, ref_data)
                        } else {
                            (lhs_data.obs * lambda_j, ref_data * lambda_i)
                        }
                    },
                };

                let value = match combination {
                    Combination::NarrowLane => alpha * beta * (v_i + gamma * v_j),
                    _ => alpha * beta * (v_i - gamma * v_j),
                };

                let combination = (lhs_observable.clone(), ref_observable.clone());
                if let Some(data) = ret.get_mut(&combination) {
                    if let Some(data) = data.get_mut(&sv) {
                        data.insert(*epoch, value);
                    } else {
                        let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                        map.insert(*epoch, value);
                        data.insert(*sv, map);
                    }
                } else {
                    let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                    map.insert(*epoch, value);
                    let mut bmap: BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>> = BTreeMap::new();
                    bmap.insert(*sv, map);
                    ret.insert(combination, bmap);
                }
            }
        }
    }
    ret
}

#[cfg(feature = "obs")]
fn mw_combination(
    rec: &Record,
) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
    let code_narrow = dual_freq_combination(rec, Combination::NarrowLane);
    let mut phase_wide = dual_freq_combination(rec, Combination::WideLane);

    phase_wide.retain(|(lhs_obs, rhs_obs), phase_wide| {
        let lhs_code_obs =
            Observable::from_str(&format!("C{}", &lhs_obs.to_string()[1..])).unwrap();
        let rhs_code_obs =
            Observable::from_str(&format!("C{}", &rhs_obs.to_string()[1..])).unwrap();

        if lhs_obs.is_phase_observable() {
            if let Some(code_data) = code_narrow.get(&(lhs_code_obs, rhs_code_obs)) {
                phase_wide.retain(|sv, phase_data| {
                    if let Some(code_data) = code_data.get(&sv) {
                        phase_data.retain(|epoch, _| code_data.get(&epoch).is_some());
                        phase_data.len() > 0
                    } else {
                        false
                    }
                });
                phase_wide.len() > 0
            } else {
                false
            }
        } else {
            false
        }
    });

    for ((lhs_obs, rhs_obs), phase_data) in phase_wide.iter_mut() {
        let lhs_code_obs =
            Observable::from_str(&format!("C{}", &lhs_obs.to_string()[1..])).unwrap();
        let rhs_code_obs =
            Observable::from_str(&format!("C{}", &rhs_obs.to_string()[1..])).unwrap();

        if let Some(code_data) = code_narrow.get(&(lhs_code_obs, rhs_code_obs)) {
            for (phase_sv, data) in phase_data {
                if let Some(code_data) = code_data.get(&phase_sv) {
                    for (epoch, phase_wide) in data {
                        if let Some(narrow_code) = code_data.get(&epoch) {
                            *phase_wide -= narrow_code;
                        }
                    }
                }
            }
        }
    }
    phase_wide
}

#[cfg(feature = "obs")]
impl Combine for Record {
    fn combine(
        &self,
        c: Combination,
    ) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
        match c {
            Combination::GeometryFree
            | Combination::IonosphereFree
            | Combination::NarrowLane
            | Combination::WideLane => dual_freq_combination(self, c),
            Combination::MelbourneWubbena => mw_combination(self),
        }
    }
}

#[cfg(feature = "obs")]
use crate::{
    carrier,
    observation::Dcb, //Mp},
};

#[cfg(feature = "obs")]
impl Dcb for Record {
    fn dcb(&self) -> HashMap<String, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
        let mut ret: HashMap<String, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> =
            HashMap::new();
        for (epoch, (_, vehicles)) in self {
            for (sv, observations) in vehicles {
                for (lhs_observable, lhs_observation) in observations {
                    if !lhs_observable.is_phase_observable()
                        && !lhs_observable.is_pseudorange_observable()
                    {
                        continue;
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

                                    for (op, vehicles) in ret.iter_mut() {
                                        if op.contains(lhs_code) {
                                            already_diffd = true;

                                            // determine this code's role in the diff op
                                            // so it remains consistent
                                            let items: Vec<&str> = op.split('-').collect();

                                            if lhs_code == items[0] {
                                                // code is differenced
                                                if let Some(data) = vehicles.get_mut(sv) {
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
                                                    vehicles.insert(*sv, bmap);
                                                }
                                            } else {
                                                // code is refered to
                                                if let Some(data) = vehicles.get_mut(sv) {
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
                                                    vehicles.insert(*sv, bmap);
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
                                        let mut map: BTreeMap<
                                            SV,
                                            BTreeMap<(Epoch, EpochFlag), f64>,
                                        > = BTreeMap::new();
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn obs_record_is_new_epoch() {
        assert!(is_new_epoch(
            "95 01 01 00 00 00.0000000  0  7 06 17 21 22 23 28 31",
            Version { major: 2, minor: 0 }
        ));
        assert!(!is_new_epoch(
            "21700656.31447  16909599.97044          .00041  24479973.67844  24479975.23247",
            Version { major: 2, minor: 0 }
        ));
        assert!(is_new_epoch(
            "95 01 01 11 00 00.0000000  0  8 04 16 18 19 22 24 27 29",
            Version { major: 2, minor: 0 }
        ));
        assert!(!is_new_epoch(
            "95 01 01 11 00 00.0000000  0  8 04 16 18 19 22 24 27 29",
            Version { major: 3, minor: 0 }
        ));
        assert!(is_new_epoch(
            "> 2022 01 09 00 00 30.0000000  0 40",
            Version { major: 3, minor: 0 }
        ));
        assert!(!is_new_epoch(
            "> 2022 01 09 00 00 30.0000000  0 40",
            Version { major: 2, minor: 0 }
        ));
        assert!(!is_new_epoch(
            "G01  22331467.880   117352685.28208        48.950    22331469.28",
            Version { major: 3, minor: 0 }
        ));
    }
}

/*
 * Code multipath bias
 */
#[cfg(feature = "obs")]
pub(crate) fn code_multipath(
    rec: &Record,
) -> HashMap<Observable, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
    let mut ret: HashMap<Observable, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> =
        HashMap::new();

    for (epoch, (_, vehicles)) in rec {
        for (sv, observations) in vehicles {
            for (observable, obsdata) in observations {
                if !observable.is_pseudorange_observable() {
                    continue;
                }

                let code = observable.to_string();
                let carrier = &code[1..2].to_string();
                let code_is_l1 = code.contains("1");

                let mut phase_i = Option::<f64>::None;
                let mut phase_j = Option::<f64>::None;
                let mut f_i = Option::<f64>::None;
                let mut f_j = Option::<f64>::None;

                for (rhs_observable, rhs_data) in observations {
                    if !rhs_observable.is_phase_observable() {
                        continue;
                    }
                    let rhs_code = rhs_observable.to_string();

                    // identify carrier signal
                    let rhs_carrier = Carrier::from_observable(sv.constellation, &rhs_observable);
                    if rhs_carrier.is_err() {
                        continue;
                    }
                    let rhs_carrier = rhs_carrier.unwrap();
                    let lambda = rhs_carrier.wavelength();

                    if code_is_l1 {
                        if rhs_code.contains("2") {
                            f_j = Some(rhs_carrier.frequency());
                            phase_j = Some(rhs_data.obs * lambda);
                        } else if rhs_code.contains(carrier) {
                            f_i = Some(rhs_carrier.frequency());
                            phase_i = Some(rhs_data.obs * lambda);
                        }
                    } else {
                        if rhs_code.contains("1") {
                            f_j = Some(rhs_carrier.frequency());
                            phase_j = Some(rhs_data.obs * lambda);
                        } else if rhs_code.contains(carrier) {
                            f_i = Some(rhs_carrier.frequency());
                            phase_i = Some(rhs_data.obs * lambda);
                        }
                    }

                    if phase_i.is_some() && phase_j.is_some() {
                        break; // DONE
                    }
                }

                if phase_i.is_none() || phase_j.is_none() {
                    continue; // can't proceed
                }

                let gamma = (f_i.unwrap() / f_j.unwrap()).powi(2);
                let alpha = (gamma + 1.0) / (gamma - 1.0);
                let beta = 2.0 / (gamma - 1.0);
                let value = obsdata.obs - alpha * phase_i.unwrap() + beta * phase_j.unwrap();

                if let Some(data) = ret.get_mut(&observable) {
                    if let Some(data) = data.get_mut(&sv) {
                        data.insert(*epoch, value);
                    } else {
                        let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                        map.insert(*epoch, value);
                        data.insert(*sv, map);
                    }
                } else {
                    let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                    map.insert(*epoch, value);
                    let mut bmap: BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>> = BTreeMap::new();
                    bmap.insert(*sv, map);
                    ret.insert(observable.clone(), bmap);
                }
            }
        }
    }
    ret
}
