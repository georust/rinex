//! Observation RINEX parsing
use crate::{
    epoch::{parse_in_timescale as parse_epoch_in_timescale, parse_utc as parse_utc_epoch},
    observation::ParsingError,
    prelude::{
        ClockObservation, Constellation, EpochFlag, Header, LliFlags, ObsKey, Observable,
        Observations, SignalObservation, TimeScale, Version, SNR, SV,
    },
};

use std::{
    collections::HashMap,
    str::{FromStr, Lines},
};

// TODO: see if we can get rid of num_integer
use num_integer::div_ceil;

#[cfg(feature = "log")]
use log::{debug, error};

/// Returns true if given content matches a new OBSERVATION data epoch
pub fn is_new_epoch(line: &str, v: Version) -> bool {
    if v.major < 3 {
        if line.len() < 30 {
            false
        } else {
            // SPLICE flag handling (still an Observation::flag)
            let significant = !line[0..26].trim().is_empty();
            let epoch = parse_utc_epoch(&line[0..26]);
            let flag = EpochFlag::from_str(line[26..29].trim());
            if significant {
                epoch.is_ok() && flag.is_ok()
            } else if flag.is_err() {
                false
            } else {
                match flag.unwrap() {
                    EpochFlag::AntennaBeingMoved
                    | EpochFlag::NewSiteOccupation
                    | EpochFlag::HeaderInformationFollows
                    | EpochFlag::ExternalEvent => true,
                    _ => false,
                }
            }
        }
    } else {
        // Modern RINEX has a simple marker, like all V4 modern files
        match line.chars().next() {
            Some(c) => {
                c == '>' // epochs always delimited
                         // by this new identifier
            },
            _ => false,
        }
    }
}

/// Parses all [Record] entries from readable content.
/// ## Inputs
///   - header: reference to [Header] specs
///   - content: readable content
///   - ts: file [TimeScale] specs
///   - observations: actual results, allocated only once
///   that should be reset before this call
pub fn parse_epoch(
    header: &Header,
    content: &str,
    ts: TimeScale,
    observations: &mut Observations,
) -> Result<ObsKey, ParsingError> {
    let mut lines = content.lines();

    let mut line = match lines.next() {
        Some(l) => l,
        _ => return Err(ParsingError::EmptyEpoch),
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

    let (num_sat, rem) = rem.split_at(3);
    let num_sat = num_sat
        .trim()
        .parse::<u16>()
        .map_err(|_| ParsingError::NumSatParsing)?;

    let key = ObsKey { epoch, flag };

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
    if let Some(offset) = offs {
        if let Ok(offset_s) = offset.parse::<f64>() {
            if let Some(ref mut clock) = observations.clock {
                clock.set_offset_s(epoch, offset_s);
            } else {
                observations.with_clock_observation(
                    ClockObservation::default().with_offset_s(epoch, offset_s),
                );
            }
        }
    }

    match flag {
        EpochFlag::Ok | EpochFlag::PowerFailure | EpochFlag::CycleSlip => {
            parse_observations(header, key, num_sat, rem, lines, observations)?;
        },
        _ => {
            // TODO following OBS_RINEX refactor:
            // Epoch events are not supported yet (coming soong)
            // parse_event(header, epoch, flag, n_sat, clock_offset, rem, lines)
        },
    }

    Ok(key)
}

fn parse_observations(
    header: &Header,
    key: ObsKey,
    num_sat: u16,
    rem: &str,
    mut lines: Lines<'_>,
    observations: &mut Observations,
) -> Result<(), ParsingError> {
    // previously identified observables (that we expect)
    let obs = header.obs.as_ref().unwrap();
    let observables = &obs.codes;

    let constellation = header.constellation;

    match header.version.major {
        2 => {
            // Sets the satellite systems description, which consits in
            //  - end of current line
            //  - possible following lines
            let mut systems_str = String::with_capacity(24 * 3); //SVNN
            systems_str.push_str(rem.trim());
            while systems_str.len() / 3 < num_sat.into() {
                if let Some(l) = lines.next() {
                    systems_str.push_str(l.trim());
                } else {
                    return Err(ParsingError::BadV2SatellitesDescription);
                }
            }
            parse_observations_v2(
                &systems_str,
                constellation,
                observables,
                lines,
                observations,
            );
        },
        _ => {
            parse_observations_v3(observables, lines, observations);
        },
    }

    Ok(())
}

/// Parses all [Observations] as described by following V2 content.
/// Old format is tedious:
///   - vehicle description is contained in first line
///   - wrapped in as many lines as needed
/// Inputs
///   - system_str: first line description
///   - constellation: [Constellation] specs defined in [Header]
///   - observables: reference to [Observable]s specs defined in [Header]
///   - lines: remaining [Lines] Iterator
fn parse_observations_v2(
    systems_str: &str,
    head_constellation: Option<Constellation>,
    head_observables: &HashMap<Constellation, Vec<Observable>>,
    lines: Lines<'_>,
    observations: &mut Observations,
) {
    const SVNN_SIZE: usize = 3; // SVNN standard
    const MAX_OBSERVABLES_LINE: usize = 5; // max in a single line
    const OBSERVABLE_F14_WIDTH: usize = 14;
    const OBSERVABLE_WIDTH: usize = OBSERVABLE_F14_WIDTH + 2; // data +lli +snr +1separator
    const MIN_LINE_WIDTH: usize = 10; // below 10 bytes, we're sure this line is empty (=not a single obs)

    // basic check that avoid entering the loop for nothing
    if systems_str.len() < SVNN_SIZE {
        // does not look good (=rubbish first line)
        #[cfg(feature = "log")]
        error!("abort: empty content");
        return;
    }

    #[cfg(feature = "log")]
    debug!("V2 parsing: \"{}\"", systems_str);

    // sv pointer
    let mut sv = SV::default();
    let mut sv_ptr = 0;
    let mut sv_identified = false;

    // observable pointer
    let mut obs_ptr = 0;
    let mut obs_identified = false;
    let mut observables = &Vec::<Observable>::default();

    // browse all lines
    for line in lines {
        // [SV] identification
        //  1. on first line (not defined yet!)
        //  2. every time past SV is concluded
        if !sv_identified {
            // identify new SV
            let start = sv_ptr * SVNN_SIZE;
            let system = &systems_str[start..start + SVNN_SIZE].trim();

            // actual parsing
            if let Ok(found) = SV::from_str(system) {
                sv = found;
            } else {
                // This may actually fail,
                // The ""great""" V2 mono constellation which may omit X in XYY descrption
                match head_constellation {
                    Some(Constellation::Mixed) | None => {
                        #[cfg(feature = "log")]
                        error!("abort: no constellation specs in header");
                        break;
                    },
                    Some(gnss) => {
                        if let Ok(prn) = system.trim().parse::<u8>() {
                            sv = SV {
                                prn,
                                constellation: gnss,
                            };
                        } else {
                            #[cfg(feature = "log")]
                            error!("abort: invalid svnn descriptor");
                            break;
                        }
                    },
                }
            }

            // move on to next
            sv_ptr += SVNN_SIZE;
            sv_identified = true;
            obs_identified = false;
        }

        // [Observable] identification
        //  - locate [Observable]s specs for this [SV]
        if !obs_identified {
            observables = if sv.constellation.is_sbas() {
                if let Some(observables) = head_observables.get(&Constellation::SBAS) {
                    observables
                } else {
                    #[cfg(feature = "log")]
                    error!("abort (sbas): not observable specs");
                    break;
                }
            } else {
                if let Some(observables) = head_observables.get(&sv.constellation) {
                    observables
                } else {
                    #[cfg(feature = "log")]
                    error!("abort ({}): no observable specs", sv.constellation);
                    break;
                }
            };

            obs_ptr = 0;
            obs_identified = true;

            #[cfg(feature = "log")]
            debug!("{}: {:?}", sv, observables);
        }

        let line_width = line.len();

        if line_width < MIN_LINE_WIDTH {
            #[cfg(feature = "log")]
            debug!("empty line: \"{}\"", line);

            // => increment by maximal number of vehicles that may remain
            obs_ptr += MAX_OBSERVABLES_LINE;

            // this concludes this vehicle
            if obs_ptr >= observables.len() {
                sv_identified = false;
                obs_identified = false;
            }

            continue;
        }

        // num obs contained this line
        let num_obs = div_ceil(line_width, OBSERVABLE_WIDTH);

        // got something to parse (= at least 1 OBS)
        #[cfg(feature = "log")]
        debug!("line: \"{}\" [={}]", line, num_obs);

        // process all of them
        for i in 0..num_obs {
            if obs_ptr > observables.len() {
                // line is abnormally long (trailing whitespaces): abort
                break;
            }

            let start = obs_ptr * OBSERVABLE_WIDTH;
            let end = (start + OBSERVABLE_WIDTH).min(line_width);
            let slice = &line[start..end];

            #[cfg(feature = "log")]
            debug!("observation: \"{}\"", slice);

            // parse possible LLI
            let mut lli = Option::<LliFlags>::None;

            if slice.len() > OBSERVABLE_F14_WIDTH {
                let lli_slice = &slice[OBSERVABLE_F14_WIDTH..OBSERVABLE_F14_WIDTH + 1];
                match lli_slice.parse::<u8>() {
                    Ok(unsigned) => {
                        lli = LliFlags::from_bits(unsigned);
                    },
                    Err(e) => {
                        #[cfg(feature = "log")]
                        error!("lli: {}", e);
                    },
                }
            }

            // parse possible SNR
            let mut snr = Option::<SNR>::None;

            if slice.len() > OBSERVABLE_F14_WIDTH + 1 {
                let snr_slice = &slice[OBSERVABLE_F14_WIDTH + 1..OBSERVABLE_F14_WIDTH + 2];
                match SNR::from_str(snr_slice) {
                    Ok(found) => {
                        snr = Some(found);
                    },
                    Err(e) => {
                        #[cfg(feature = "log")]
                        error!("snr: {:?}", e);
                    },
                }
            }

            // parse observed value
            let end: usize = slice.len().min(OBSERVABLE_F14_WIDTH);
            let value_slice = &slice[0..end];
            if let Ok(value) = value_slice.trim().parse::<f64>() {
                observations.signals.push(SignalObservation {
                    sv,
                    value,
                    snr,
                    lli,
                    observable: observables[obs_ptr].clone(),
                });
            }

            obs_ptr += 1;

            if obs_ptr >= observables.len() {
                sv_identified = false;
                obs_identified = false;

                #[cfg(feature = "log")]
                debug!("{} completed", sv);
            } else {
                #[cfg(feature = "log")]
                debug!("{}/{}", obs_ptr, num_obs);
            }
        }
    } // for all lines provided
}

/// Parses all [Observation]s described by following [Lines].
fn parse_observations_v3(
    head_observables: &HashMap<Constellation, Vec<Observable>>,
    lines: Lines<'_>,
    observations: &mut Observations,
) {
    const SVNN_SIZE: usize = 3;
    const OBSERVABLE_F14_WIDTH: usize = 14;
    const OBSERVABLE_WIDTH: usize = 16; // F14 +2 flags

    let mut sv = SV::default(); // single alloc

    // browse all lines
    for line in lines {
        #[cfg(feature = "log")]
        debug!("line: \"{}\"", line);

        // identify SV
        let sv_str = &line[0..SVNN_SIZE];
        match SV::from_str(sv_str) {
            Ok(found) => {
                sv = found;
            },
            Err(e) => {
                #[cfg(feature = "log")]
                error!("sv parsing: {}", e);
                continue;
            },
        }

        // identify [Observable]s
        let observables = if sv.constellation.is_sbas() {
            head_observables.get(&Constellation::SBAS)
        } else {
            head_observables.get(&sv.constellation)
        };

        if observables.is_none() {
            #[cfg(feature = "log")]
            error!("{}: on observable specs", sv);
            continue;
        }

        let observables = observables.unwrap();

        #[cfg(feature = "log")]
        debug!("{}: {:?}", sv, observables);

        let num_obs = line.len() / OBSERVABLE_WIDTH;
        let mut obs_ptr = 0;
        let mut offset = SVNN_SIZE + 1;

        for i in 0..num_obs {
            if i == observables.len() {
                // line is abnormally long (trailing whitespaces)
                break;
            }

            let end = (offset + OBSERVABLE_WIDTH).min(line.len());
            let slice = &line[offset..end];

            let mut lli = Option::<LliFlags>::None;

            if slice.len() > OBSERVABLE_F14_WIDTH {
                let start = offset + OBSERVABLE_F14_WIDTH;
                let lli_slice = &line[start..start + 1];

                match lli_slice.parse::<u8>() {
                    Ok(unsigned) => {
                        lli = LliFlags::from_bits(unsigned);
                    },
                    Err(e) => {
                        #[cfg(feature = "log")]
                        error!("lli: {}", e);
                    },
                }
            }

            let mut snr = Option::<SNR>::None;

            if slice.len() > OBSERVABLE_F14_WIDTH + 1 {
                let start = offset + OBSERVABLE_F14_WIDTH + 1;
                let snr_slice = &line[start..start + 1];

                match SNR::from_str(snr_slice) {
                    Ok(found) => {
                        snr = Some(found);
                    },
                    Err(e) => {
                        #[cfg(feature = "log")]
                        error!("snr: {:?}", e);
                    },
                }
            }

            let end = (offset + OBSERVABLE_F14_WIDTH).min(slice.len());

            if let Ok(value) = slice[offset..offset + end].parse::<f64>() {
                observations.signals.push(SignalObservation {
                    sv,
                    value,
                    lli,
                    snr,
                    observable: observables[obs_ptr].clone(),
                });
            }
        }
    } //browse all lines
}

#[cfg(test)]
mod test {
    use super::is_new_epoch;
    use crate::prelude::Version;
    #[test]
    fn test_new_epoch() {
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
