use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

use itertools::Itertools;

use crate::{
    epoch::{
        parse_in_timescale as parse_epoch_in_timescale,
        ParsingError as EpochParsingError,
    },
    merge::{Error as MergeError, Merge},
    observation::{Observation, SignalObservation, flag::Error as FlagError, EpochFlag, SNR, LliFlags},
    prelude::{Epoch, Header, SV, Duration, Constellation, TimeScale},
    split::{Error as SplitError, Split},
    types::Type,
    Carrier, Observable,
};

#[cfg(feature = "processing")]
use qc_traits::processing::{
    DecimationFilter, DecimationFilterType, FilterItem, MaskFilter, MaskOperand, Repair,
};

/// One file Epoch describes several [Observation]s
/// ## Inputs
///   - header: closely tied to [Header] definitions
///   - ts: [TimeScale] dependent
///   - content: actual description
pub(crate) fn parse_epoch(
    header: &Header,
    content: &str,
    ts: TimeScale,
) -> Result<Vec<Observation>, Error> {

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

    let (date, rem) = line.split_at(offset);
    let epoch = parse_epoch_in_timescale(date, ts)?;
    let (flag, rem) = rem.split_at(3);
    let flag = EpochFlag::from_str(flag.trim())?;
    let (n_sat, rem) = rem.split_at(3);
    let n_sat = n_sat.trim().parse::<u16>()?;

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

    match flag {
        EpochFlag::Ok | EpochFlag::PowerFailure | EpochFlag::CycleSlip => {
            parse_normal(header, epoch, flag, n_sat, clock_offset, rem, lines)
        },
        _ => parse_event(header, epoch, flag, n_sat, clock_offset, rem, lines),
    }
}

fn parse_normal(
    header: &Header,
    epoch: Epoch,
    flag: EpochFlag,
    n_sat: u16,
    clock_offset: Option<f64>,
    rem: &str,
    mut lines: std::str::Lines<'_>,
) -> Result<Vec<(ObsKey, Observation)>, Error> {
    // previously identified observables (that we expect)
    let obs = header.obs.as_ref().unwrap();
    let observables = &obs.codes;
    match header.version.major {
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
        _ => Ok(parse_v3(epoch, flag, observables, lines)),
    }
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
) -> Vec<(ObsKey, Observation)> {
    let svnn_size = 3; // SVNN standard
    let nb_max_observables = 5; // in a single line
    let observable_width = 16; // data + 2 flags + 1 whitespace
    let mut sv_ptr = 0; // svnn pointer
    let mut obs_ptr = 0; // observable pointer
    let mut sv = SV::default();
    let mut observables: &Vec<Observable>;

    let mut key = ObsKey::default();
    let mut ret = Vec::<(ObsKey, Observation)>::with_capacity(8);
    //println!("{:?}", header_observables); // DEBUG
    //println!("\"{}\"", systems); // DEBUG

    // parse first system we're dealing with
    if systems.len() < svnn_size {
        // Can't even parse a single vehicle;
        // epoch descriptor is totally corrupt, stop here
        return Default::default();
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
                        return ret;
                    }
                }
            },
            None => return ret,
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
                return ret;
            }
        },
        false => {
            if let Some(observables) = header_observables.get(&sv.constellation) {
                observables
            } else {
                // failed to identify observations for this vehicle
                return ret;
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
                    ret.push((key, Observation { obs, lli, snr }));
                } //f64::obs
            } // parsing all observations
            if nb_obs < nb_max_observables {
                obs_ptr += nb_max_observables - nb_obs;
            }
        }
        //println!("OBS COUNT {}", obs_ptr); //DEBUG

        if obs_ptr >= observables.len() {
            // we're done with current vehicle
            obs_ptr = 0;
            //identify next vehicle
            if sv_ptr >= systems.len() {
                // last vehicle
                return ret;
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
                                return ret;
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
                        return ret;
                    }
                },
                false => {
                    if let Some(observables) = header_observables.get(&sv.constellation) {
                        observables
                    } else {
                        // failed to identify observations for this vehicle
                        return ret;
                    }
                },
            };
            //println!("{:?}", observables); // DEBUG
        }
    } // for all lines provided
    ret
}

/*
 * Parses a V3 epoch from given lines iteratoor
 * Format is much simpler, one vehicle is described in a single line
 */
fn parse_v3(
    epoch: Epoch,
    flag: EpochFlag,
    observables: &HashMap<Constellation, Vec<Observable>>,
    lines: std::str::Lines<'_>,
) -> Vec<(ObsKey, Observation)> {
    let svnn_size = 3; // SVNN standard
    let observable_width = 16; // data + 2 flags

    let mut sv = SV::default();
    let mut key = ObsKey::default();
    let mut observable = Observable::default();
    let mut ret = Vec::<(ObsKey, Observation)>::new();

    for line in lines {
        // browse all lines
        //println!("parse_v3: \"{}\"", line); //DEBUG
        let (sv, line) = line.split_at(svnn_size);
        if let Ok(sv) = SV::from_str(sv) {
            let observables = match sv.constellation.is_sbas() {
                true => observables.get(&Constellation::SBAS),
                false => observables.get(&sv.constellation),
            };
            //println!("SV: {} OBSERVABLES: {:?}", sv, observables); // DEBUG
            if let Some(observables) = observables {
                let nb_obs = line.len() / observable_width;
                let mut rem = line;
                for i in 0..nb_obs {
                    if i == observables.len() {
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
                    if let Ok(value) = obs.trim().parse::<f64>() {
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
                            }observable
                        }
                        //println!("LLI {:?}", lli); //DEBUG
                        //println!("SSI {:?}", snr);
                        // build content
                        let key = ObsKey {
                            sv,
                            epoch,
                            flag,
                        };
                        let value = Observation::new_signal(value, snr, lli);
                        ret.push((key, value));
                    }
                }
                if rem.len() >= observable_width - 2 {
                    let mut snr: Option<SNR> = None;
                    let mut lli: Option<LliFlags> = None;
                    let obs = &rem[0..observable_width - 2];
                    if let Ok(value) = obs.trim().parse::<f64>() {
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
                        let key = ObsKey {
                            sv,
                            epoch,
                            flag,
                            observable: observables[nb_obs].clone(),
                        };
                        let value = Observation::new_signal(value, snr, lli);
                        ret.push((key, value));
                    }
                }
            } //observables
        } // SV::from_str
    } //lines
    ret
}

#[cfg(feature = "processing")]
fn repair_zero_mut(rec: &mut Record) {
    rec.retain(|_, meas| meas.value != 0.0);
}

#[cfg(feature = "processing")]
fn carrier_phase_cycles_mut(rec: &mut Record) {
    for (key, obs) in rec.iter_mut() {
        if key.observable.is_phase_observable() {
            if let Ok(carrier) = key.observable.carrier(sv.constellation) {
                obs.value *= carrier.wavelength();
            }
        }
    }
}

#[cfg(feature = "processing")]
fn null_phase_origin_mut(rec: &mut Record) {
    let mut t0 = HashMap::<(SV, Observable), f64>::new();
    for (key, obs) in rec.iter_mut() {
        if key.observable.is_phase_observable() {
            if let Some(t0) = t0.get(&(key.sv, key.observable.clone())) {
                obs.value -= t0;
            } else {
                t0.insert((key.sv, key.observable.clone()), obs.value);
                obs.value = 0.0_f64;
            }
        }
    }
}

#[cfg(feature = "processing")]
pub(crate) fn repair_mut(rec: &mut Record, repair: Repair) {
    match repair {
        Repair::Zero => repair_zero_mut(rec),
        Repair::NullPhaseOrigin => null_phase_origin_mut(rec),
        Repair::CarrierPhaseCycles => carrier_phase_cycles_mut(rec),
    }
}

#[cfg(feature = "processing")]
pub(crate) fn observation_mask_mut(rec: &mut Record, mask: &MaskFilter) {
    match mask.operand {
        MaskOperand::Equals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|(e, _), _| *e == *epoch),
            FilterItem::ClockItem => {
                rec.retain(|_, (clk, _)| clk.is_some());
            },
            FilterItem::ConstellationItem(constells) => {
                let mut broad_sbas_filter = false;
                for c in constells {
                    broad_sbas_filter |= *c == Constellation::SBAS;
                }
                rec.retain(|_, (_, svs)| {
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
            FilterItem::SvItem(items) => {
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| items.contains(sv));
                    !svs.is_empty()
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, (_, svs)| {
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
            FilterItem::ComplexItem(filter) => {
                // try to interprate as [Observable]
                let observables = filter
                    .iter()
                    .filter_map(|f| {
                        if let Ok(ob) = Observable::from_str(f) {
                            Some(ob)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                if observables.len() > 0 {
                    rec.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|ob, _| observables.contains(&ob));
                            !obs.is_empty()
                        });
                        !svs.is_empty()
                    });
                }
            },
            _ => {},
        },
        MaskOperand::NotEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|(e, _), _| *e != *epoch),
            FilterItem::ClockItem => {
                rec.retain(|_, (clk, _)| clk.is_none());
            },
            FilterItem::ConstellationItem(constells) => {
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| !constells.contains(&sv.constellation));
                    !svs.is_empty()
                });
            },
            FilterItem::SvItem(items) => {
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| !items.contains(sv));
                    !svs.is_empty()
                });
            },
            FilterItem::ComplexItem(filter) => {
                // try to interprate as [Observable]
                let observables = filter
                    .iter()
                    .filter_map(|f| {
                        if let Ok(ob) = Observable::from_str(f) {
                            Some(ob)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                if observables.len() > 0 {
                    rec.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|ob, _| !observables.contains(&ob));
                            !obs.is_empty()
                        });
                        !svs.is_empty()
                    });
                }
            },
            _ => {},
        },
        MaskOperand::GreaterEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|(e, _), _| *e >= *epoch),
            FilterItem::SvItem(items) => {
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| {
                        let mut retain = true;
                        for item in items {
                            if item.constellation == sv.constellation {
                                retain = sv.prn >= item.prn;
                            }
                        }
                        retain
                    });
                    !svs.is_empty()
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, (_, svs)| {
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
        MaskOperand::GreaterThan => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|(e, _), _| *e > *epoch),
            FilterItem::SvItem(items) => {
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| {
                        let mut retain = true;
                        for item in items {
                            if item.constellation == sv.constellation {
                                retain = sv.prn > item.prn;
                            }
                        }
                        retain
                    });
                    !svs.is_empty()
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, (_, svs)| {
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
        MaskOperand::LowerEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|(e, _), _| *e <= *epoch),
            FilterItem::SvItem(items) => {
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| {
                        let mut retain = true;
                        for item in items {
                            if item.constellation == sv.constellation {
                                retain = sv.prn <= item.prn;
                            }
                        }
                        retain
                    });
                    !svs.is_empty()
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, (_, svs)| {
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
        MaskOperand::LowerThan => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|(e, _), _| *e < *epoch),
            FilterItem::SvItem(items) => {
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| {
                        let mut retain = true;
                        for item in items {
                            if item.constellation == sv.constellation {
                                retain = sv.prn < item.prn;
                            }
                        }
                        retain
                    });
                    !svs.is_empty()
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, (_, svs)| {
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

#[cfg(feature = "processing")]
pub(crate) fn observation_decim_mut(rec: &mut Record, decim: &DecimationFilter) {
    if decim.item.is_some() {
        todo!("targetted decimation not supported yet");
    }
    match decim.filter {
        DecimationFilterType::Modulo(r) => {
            let mut i = 0;
            rec.retain(|_, _| {
                let retained = (i % r) == 0;
                i += 1;
                retained
            });
        },
        DecimationFilterType::Duration(interval) => {
            let mut last_retained = Option::<Epoch>::None;
            rec.retain(|(e, _), _| {
                if let Some(last) = last_retained {
                    let dt = *e - last;
                    if dt >= interval {
                        last_retained = Some(*e);
                        true
                    } else {
                        false
                    }
                } else {
                    last_retained = Some(*e);
                    true // always retain 1st epoch
                }
            });
        },
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
                let lhs_is_l1 = lhs_code.contains('1');
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
                    if refcode.contains('1') {
                        reference = Some((ref_observable.clone(), ref_data.obs));
                        break; // DONE searching
                    }
                }

                if reference.is_none() {
                    continue; // can't proceed further
                }
                let (ref_observable, ref_data) = reference.unwrap();

                // determine frequencies
                let lhs_carrier = Carrier::from_observable(sv.constellation, lhs_observable);
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
                    Combination::IonosphereFree => fi.powi(2),
                    Combination::WideLane | Combination::NarrowLane => fi,
                    Combination::MelbourneWubbena => unreachable!("mw combination"),
                };

                let gamma = match combination {
                    Combination::GeometryFree => 1.0_f64,
                    Combination::IonosphereFree => fj.powi(2),
                    Combination::WideLane | Combination::NarrowLane => fj,
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
                    Combination::NarrowLane => alpha * (beta * v_i + gamma * v_j),
                    _ => alpha * (beta * v_i - gamma * v_j),
                };

                let combination = (lhs_observable.clone(), ref_observable.clone());
                if let Some(data) = ret.get_mut(&combination) {
                    if let Some(data) = data.get_mut(sv) {
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
                    if let Some(code_data) = code_data.get(sv) {
                        phase_data.retain(|epoch, _| code_data.get(epoch).is_some());
                        !phase_data.is_empty()
                    } else {
                        false
                    }
                });
                !phase_wide.is_empty()
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
                if let Some(code_data) = code_data.get(phase_sv) {
                    for (epoch, phase_wide) in data {
                        if let Some(narrow_code) = code_data.get(epoch) {
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

#[cfg(feature = "obs")]
pub(crate) fn lli_and_mask_mut(rec: &mut Record, mask: LliFlags) {
    rec.retain(|_, obs| {
        if let Some(lli) = obs.lli {
            lli.intersects(mask)
        } else {
            false
        }
    })
}

#[cfg(feature = "obs")]
pub(crate) fn phase_lock_loss_mut(rec: &mut Record) {
    rec.retain(|key, obs| {
        if key.observable.is_phase_observable() {
            if let Some(lli) = obs.lli {
                !lli.intersects(LliFlags::LOCK_LOSS)
            } else {
                true
            }
        } else {
            true
        }
    });
}

/*
 * Code multipath bias
 */
#[cfg(feature = "obs")]
pub(crate) fn code_multipath(rec: &Record) -> HashMap<ObsKey, Observation> {
    Default::default()
}
//    let mut ret: HashMap<Observable, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> =
//        HashMap::new();
//
//    for (epoch, (_, vehicles)) in rec {
//        for (sv, observations) in vehicles {
//            for (observable, obsdata) in observations {
//                if !observable.is_pseudorange_observable() {
//                    continue;
//                }
//
//                let code = observable.to_string();
//                let carrier = &code[1..2].to_string();
//                let code_is_l1 = code.contains('1');
//
//                let mut phase_i = Option::<f64>::None;
//                let mut phase_j = Option::<f64>::None;
//                let mut f_i = Option::<f64>::None;
//                let mut f_j = Option::<f64>::None;
//
//                for (rhs_observable, rhs_data) in observations {
//                    if !rhs_observable.is_phase_observable() {
//                        continue;
//                    }
//                    let rhs_code = rhs_observable.to_string();
//
//                    // identify carrier signal
//                    let rhs_carrier = Carrier::from_observable(sv.constellation, rhs_observable);
//                    if rhs_carrier.is_err() {
//                        continue;
//                    }
//                    let rhs_carrier = rhs_carrier.unwrap();
//                    let lambda = rhs_carrier.wavelength();
//
//                    if code_is_l1 {
//                        if rhs_code.contains('2') {
//                            f_j = Some(rhs_carrier.frequency());
//                            phase_j = Some(rhs_data.obs * lambda);
//                        } else if rhs_code.contains(carrier) {
//                            f_i = Some(rhs_carrier.frequency());
//                            phase_i = Some(rhs_data.obs * lambda);
//                        }
//                    } else if rhs_code.contains('1') {
//                        f_j = Some(rhs_carrier.frequency());
//                        phase_j = Some(rhs_data.obs * lambda);
//                    } else if rhs_code.contains(carrier) {
//                        f_i = Some(rhs_carrier.frequency());
//                        phase_i = Some(rhs_data.obs * lambda);
//                    }
//
//                    if phase_i.is_some() && phase_j.is_some() {
//                        break; // DONE
//                    }
//                }
//
//                if phase_i.is_none() || phase_j.is_none() {
//                    continue; // can't proceed
//                }
//
//                let gamma = (f_i.unwrap() / f_j.unwrap()).powi(2);
//                let alpha = (gamma + 1.0) / (gamma - 1.0);
//                let beta = 2.0 / (gamma - 1.0);
//                let value = obsdata.obs - alpha * phase_i.unwrap() + beta * phase_j.unwrap();
//
//                if let Some(data) = ret.get_mut(observable) {
//                    if let Some(data) = data.get_mut(sv) {
//                        data.insert(*epoch, value);
//                    } else {
//                        let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
//                        map.insert(*epoch, value);
//                        data.insert(*sv, map);
//                    }
//                } else {
//                    let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
//                    map.insert(*epoch, value);
//                    let mut bmap: BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>> = BTreeMap::new();
//                    bmap.insert(*sv, map);
//                    ret.insert(observable.clone(), bmap);
//                }
//            }
//        }
//    }
//    ret
//}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        header::Header,
        version::Version,
        observation::Observation,
        epoch::parse_utc as parse_utc_epoch,
        prelude::{TimeScale, SV},
    };

    fn parse_and_format_helper(ver: Version, epoch_str: &str, expected_flag: EpochFlag) {
        let first = parse_utc_epoch("2020 01 01 00 00  0.1000000").unwrap();
        let data: BTreeMap<SV, HashMap<Observable, Observation>> = BTreeMap::new();
        let header = Header::default()
            .with_version(ver)
            .with_observation_fields(HeaderFields::default().with_time_of_first_obs(first));
        let ts = TimeScale::UTC;
        let clock_offset: Option<f64> = None;

        let e = parse_epoch(&header, epoch_str, ts);

        match expected_flag {
            EpochFlag::Ok | EpochFlag::PowerFailure | EpochFlag::CycleSlip => {
                assert!(e.is_ok())
            },
            _ => {
                // TODO: Update alongside parse_event
                assert!(e.is_err());
                return;
            },
        }
        let ((e, flag), _, _) = e.unwrap();
        assert_eq!(flag, expected_flag);
        if ver.major < 3 {
            assert_eq!(
                fmt_epoch_v2(e, flag, &clock_offset, &data, &header)
                    .lines()
                    .next()
                    .unwrap(),
                epoch_str
            );
        } else {
            assert_eq!(
                fmt_epoch_v3(e, flag, &clock_offset, &data, &header)
                    .lines()
                    .next()
                    .unwrap(),
                epoch_str
            );
        }
    }

    #[test]
    fn obs_v2_parse_and_format() {
        parse_and_format_helper(
            Version { major: 2, minor: 0 },
            " 21 12 21  0  0 30.0000000  0  0",
            EpochFlag::Ok,
        );
        parse_and_format_helper(
            Version { major: 2, minor: 0 },
            " 21 12 21  0  0 30.0000000  1  0",
            EpochFlag::PowerFailure,
        );
        parse_and_format_helper(
            Version { major: 2, minor: 0 },
            " 21 12 21  0  0 30.0000000  2  0",
            EpochFlag::AntennaBeingMoved,
        );
        parse_and_format_helper(
            Version { major: 2, minor: 0 },
            " 21 12 21  0  0 30.0000000  3  0",
            EpochFlag::NewSiteOccupation,
        );
        parse_and_format_helper(
            Version { major: 2, minor: 0 },
            " 21 12 21  0  0 30.0000000  4  0",
            EpochFlag::HeaderInformationFollows,
        );
        parse_and_format_helper(
            Version { major: 2, minor: 0 },
            " 21 12 21  0  0 30.0000000  5  0",
            EpochFlag::ExternalEvent,
        );
        parse_and_format_helper(
            Version { major: 2, minor: 0 },
            " 21 12 21  0  0 30.0000000  6  0",
            EpochFlag::CycleSlip,
        );
    }
    #[test]
    fn obs_v3_parse_and_format() {
        parse_and_format_helper(
            Version { major: 3, minor: 0 },
            "> 2021 12 21 00 00 30.0000000  0  0",
            EpochFlag::Ok,
        );
        parse_and_format_helper(
            Version { major: 3, minor: 0 },
            "> 2021 12 21 00 00 30.0000000  1  0",
            EpochFlag::PowerFailure,
        );
        parse_and_format_helper(
            Version { major: 3, minor: 0 },
            "> 2021 12 21 00 00 30.0000000  2  0",
            EpochFlag::AntennaBeingMoved,
        );
        parse_and_format_helper(
            Version { major: 3, minor: 0 },
            "> 2021 12 21 00 00 30.0000000  3  0",
            EpochFlag::NewSiteOccupation,
        );
        parse_and_format_helper(
            Version { major: 3, minor: 0 },
            "> 2021 12 21 00 00 30.0000000  4  0",
            EpochFlag::HeaderInformationFollows,
        );
        parse_and_format_helper(
            Version { major: 3, minor: 0 },
            "> 2021 12 21 00 00 30.0000000  5  0",
            EpochFlag::ExternalEvent,
        );
        parse_and_format_helper(
            Version { major: 3, minor: 0 },
            "> 2021 12 21 00 00 30.0000000  6  0",
            EpochFlag::CycleSlip,
        );
    }
}
