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

/// Parses record entries from readable content
/// ## Input
///   - header: reference to previously parsed [Header]
///   - content: readable content
///   - ts: [TimeScale] defined by [Header]
///   - observations: preallocated [Observations] for performance issue.
/// ## Output
///   - [ObsKey] record indexer
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

    let key = ObsKey { epoch, flag };

    let (num_sat, rem) = rem.split_at(3);
    let num_sat = num_sat
        .trim()
        .parse::<u16>()
        .map_err(|_| ParsingError::NumSatParsing)?;

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
            observations
                .with_clock_observation(ClockObservation::default().with_offset_s(epoch, offset_s));
        }
    }

    match flag {
        EpochFlag::Ok | EpochFlag::PowerFailure | EpochFlag::CycleSlip => {
            parse_observations(header, num_sat, rem, lines, &mut observations.signals)?;
        },
        _ => {
            // Hardware events are not supported yet (coming soon)
            return Err(ParsingError::Event);
        },
    }

    Ok(key)
}

fn parse_observations(
    header: &Header,
    num_sat: u16,
    rem: &str,
    mut lines: std::str::Lines<'_>,
    signals: &mut Vec<SignalObservation>,
) -> Result<(), ParsingError> {
    // retrieve header specs
    let constellation = header.constellation;

    let obs = header.obs.as_ref().unwrap();

    let observables = &obs.codes;

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
            parse_signals_v2(&systems_str, constellation, observables, lines, signals);
        },
        _ => {
            parse_signals_v3(observables, lines, signals);
        },
    }

    Ok(())
}

/// Parses all [SignalObservation]s as described by following V2 content.
/// Old format is tedious:
///   - vehicle description is contained in first line
///   - wrapped in as many lines as needed
/// Inputs
///   - system_str: first line description
///   - constellation: [Constellation] specs defined in [Header]
///   - observables: reference to [Observable]s specs defined in [Header]
///   - lines: remaing [Lines] Iterator
fn parse_signals_v2(
    systems_str: &str,
    head_constellation: Option<Constellation>,
    head_observables: &HashMap<Constellation, Vec<Observable>>,
    lines: Lines<'_>,
    signals: &mut Vec<SignalObservation>,
) {
    const SVNN_SIZE: usize = 3; // SVNN standard
    const MAX_OBSERVABLES_LINE: usize = 5; // max in a single line
    const OBSERVABLE_F14_WIDTH: usize = 14;
    const OBSERVABLE_WIDTH: usize = OBSERVABLE_F14_WIDTH + 2; // data +lli +snr +1separator
    const MIN_LINE_WIDTH: usize = 1; // below 10 bytes, we're sure this line is empty (=not a single obs)

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
            let system = &systems_str[sv_ptr..sv_ptr + SVNN_SIZE].trim();

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
            println!("{}: {:?}", sv, observables);
        }

        let line_width = line.len();
        let trimmed_len = line.trim().len();

        if trimmed_len == 0 {
            #[cfg(feature = "log")]
            println!("empty line: \"{}\"", line);

            obs_ptr += MAX_OBSERVABLES_LINE;

            // this concludes this vehicle
            if obs_ptr >= observables.len() {
                sv_identified = false;
                obs_identified = false;
            }

            continue;
        }

        // num obs contained this line
        let num_obs = div_ceil(line_width, OBSERVABLE_WIDTH); //TODO: get rid of .div_ceil

        #[cfg(feature = "log")]
        println!("line: \"{}\" [={}]", line, num_obs);

        let mut offset = 0;

        // process all of them
        for _ in 0..num_obs {
            if obs_ptr > observables.len() {
                // line is abnormally long (trailing whitespaces): abort
                break;
            }

            let end = (offset + OBSERVABLE_WIDTH).min(line_width);
            let slice = &line[offset..end];

            #[cfg(feature = "log")]
            println!("observation: \"{}\" {}", slice, observables[obs_ptr]);

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
            let end = slice.len().min(OBSERVABLE_F14_WIDTH);

            if let Ok(value) = slice[..end].trim().parse::<f64>() {
                signals.push(SignalObservation {
                    sv,
                    value,
                    snr,
                    lli,
                    observable: observables[obs_ptr].clone(),
                });
            }

            obs_ptr += 1;
            offset += OBSERVABLE_F14_WIDTH + 2;

            if obs_ptr == observables.len() {
                #[cfg(feature = "log")]
                debug!("{} completed", sv);

                sv_identified = false;
                obs_identified = false;
            } else {
                #[cfg(feature = "log")]
                debug!("{}/{}", obs_ptr, num_obs);
            }
        } //num_obs

        if num_obs < MAX_OBSERVABLES_LINE {
            obs_ptr += MAX_OBSERVABLES_LINE - num_obs;
        }
    } // for all lines provided
}

/// Parses all [SignalObservation]s described by following [Lines].
fn parse_signals_v3(
    head_observables: &HashMap<Constellation, Vec<Observable>>,
    lines: Lines<'_>,
    signals: &mut Vec<SignalObservation>,
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
            error!("{}: no observable specs", sv);
            continue;
        }

        let observables = observables.unwrap();

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

            let end = OBSERVABLE_F14_WIDTH.min(slice.len());

            if let Ok(value) = slice[..end].trim().parse::<f64>() {
                signals.push(SignalObservation {
                    sv,
                    value,
                    lli,
                    snr,
                    observable: observables[i].clone(),
                });
            }

            obs_ptr += 1;
            offset += OBSERVABLE_F14_WIDTH + 2;
        }
    } //browse all lines
}

#[cfg(test)]
mod test {
    use super::{is_new_epoch, parse_epoch};
    use crate::{
        observation::{fmt_observations, HeaderFields as SpecFields},
        prelude::{
            Constellation, Epoch, EpochFlag, Header, Observable, Observations, TimeScale, Version,
            SV,
        },
        tests::toolkit::generic_observation_epoch_decoding_test,
    };
    use std::str::FromStr;

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

    #[test]
    fn test_parse_v3_1() {
        let content =
"> 2022 03 04  0  0  0.0000000  0 22        .000000000000
G01  20832393.682   109474991.854 8        49.500                    20832389.822                                                                    85305196.437 8                                                                        49.500
G03  20342516.786   106900663.487 8        50.000                    20342512.006                                                                    83299201.382 8                                                                        50.000
G04  22448754.952   117969025.322 8        48.250                    22448749.312                                                                    91923884.833 7                                                                        43.750
G06  24827263.216   130468159.526 6        39.750                    24827259.316                                                                   101663482.505 6                                                                        37.250
G09  25493930.890   133971510.403 6        41.250                    25493926.950                                                                   104393373.997 6                                                                        41.750
";
        generic_observation_epoch_decoding_test(
            content,
            3,
            Constellation::GPS,
            &[
                ("GPS", "C1C, L1C, S1C, C2P, C2W, C2S, C2L, C2X, L2P, L2W, L2S, L2L, L2X, S2P, S2W, S2S, S2L, S2X"),
            ],
            "2022-03-04T00:00:00 GPST",
            30,
            "2022-03-04T00:00:00 GPST",
            EpochFlag::Ok,
            None,
        );

        // let c1c = Observable::from_str("C1C").unwrap();
        // let l1c = Observable::from_str("L1C").unwrap();
        // let s1c = Observable::from_str("S1C").unwrap();
        // let c2w = Observable::from_str("C2W").unwrap();
        // let l2w = Observable::from_str("L2W").unwrap();
        // let s2w = Observable::from_str("S2W").unwrap();

        // let g01 = SV::from_str("G01").unwrap();
        // let g03 = SV::from_str("G03").unwrap();
        // let g04 = SV::from_str("G04").unwrap();
        // let g06 = SV::from_str("G06").unwrap();
        // let g09 = SV::from_str("G09").unwrap();

        // let signals = obs.signals.clone();

        // for sig in &signals {
        //     if sig.sv == g01 {
        //         if sig.observable == c1c {
        //             assert_eq!(sig.value, 20832393.682);
        //         } else if sig.observable == l1c {
        //             assert_eq!(sig.value, 109474991.854);
        //         } else if sig.observable == s1c {
        //             assert_eq!(sig.value, 49.500);
        //         } else if sig.observable == c2w {
        //             assert_eq!(sig.value, 20832389.822);
        //         } else if sig.observable == l2w {
        //             assert_eq!(sig.value, 85305196.437);
        //             //assert_eq!(sig.snr, Some(SNR::from(8))); //TODO
        //         } else if sig.observable == s2w {
        //             assert_eq!(sig.value, 49.500);
        //         } else {
        //             panic!("found invalid observable {}", sig.observable);
        //         }
        //     } else if sig.sv == g03 {
        //         if sig.observable == c1c {
        //             assert_eq!(sig.value, 20342516.786);
        //         } else if sig.observable == l1c {
        //             assert_eq!(sig.value, 106900663.487);
        //         } else if sig.observable == s1c {
        //             assert_eq!(sig.value, 50.0);
        //         } else if sig.observable == c2w {
        //             assert_eq!(sig.value, 20342512.006);
        //         } else if sig.observable == l2w {
        //             assert_eq!(sig.value, 83299201.382);
        //             //assert_eq!(sig.snr, Some(SNR::from(8))); //TODO
        //         } else if sig.observable == s2w {
        //             assert_eq!(sig.value, 50.000);
        //         } else {
        //             panic!("found invalid observable {}", sig.observable);
        //         }
        //     } else if sig.sv == g04 {
        //         if sig.observable == c1c {
        //             assert_eq!(sig.value, 22448754.952);
        //         } else if sig.observable == l1c {
        //             assert_eq!(sig.value, 117969025.322);
        //         } else if sig.observable == s1c {
        //             assert_eq!(sig.value, 48.250);
        //         } else if sig.observable == c2w {
        //             assert_eq!(sig.value, 22448749.312);
        //         } else if sig.observable == l2w {
        //             assert_eq!(sig.value, 91923884.833);
        //             //assert_eq!(sig.snr, Some(SNR::from(7))); //TODO
        //         } else if sig.observable == s2w {
        //             assert_eq!(sig.value, 43.750);
        //         } else {
        //             panic!("found invalid observable {}", sig.observable);
        //         }
        //     } else if sig.sv == g06 {
        //         if sig.observable == c1c {
        //             assert_eq!(sig.value, 24827263.216);
        //         } else if sig.observable == l1c {
        //             assert_eq!(sig.value, 130468159.526);
        //         } else if sig.observable == s1c {
        //             assert_eq!(sig.value, 39.750);
        //         } else if sig.observable == c2w {
        //             assert_eq!(sig.value, 24827259.316);
        //         } else if sig.observable == l2w {
        //             assert_eq!(sig.value, 101663482.505);
        //             //assert_eq!(sig.snr, Some(SNR::from(6))); //TODO
        //         } else if sig.observable == s2w {
        //             assert_eq!(sig.value, 37.250);
        //         } else {
        //             panic!("found invalid observable {}", sig.observable);
        //         }
        //     } else if sig.sv == g09 {
        //         if sig.observable == c1c {
        //             assert_eq!(sig.value, 25493930.890);
        //         } else if sig.observable == l1c {
        //             assert_eq!(sig.value, 133971510.403);
        //         } else if sig.observable == s1c {
        //             assert_eq!(sig.value, 41.250);
        //         } else if sig.observable == c2w {
        //             assert_eq!(sig.value, 25493926.950);
        //         } else if sig.observable == l2w {
        //             assert_eq!(sig.value, 104393373.997);
        //             //assert_eq!(sig.snr, Some(SNR::from(6))); //TODO
        //         } else if sig.observable == s2w {
        //             assert_eq!(sig.value, 41.75);
        //         } else {
        //             panic!("found invalid observable {}", sig.observable);
        //         }
        //     } else {
        //         panic!("invalid sv");
        //     }
        // }

        // TODO assert_eq!(formatted, content); // reciprocal
    }

    #[test]
    fn test_parse_v3_2() {
        let content =
"> 2022 03 04 00 00  0.0000000  0  9
G01  20176608.780   106028802.11808     -1009.418          50.250    20176610.080    82619851.24609      -786.562          54.500
G03  20719565.760   108882069.81508       762.203          49.750    20719566.420    84843175.68809       593.922          55.000
G04  21342618.100   112156219.39808      2167.688          48.250    21342617.440    87394474.09607      1689.105          45.500                                                                 104393373.997 6                                                                        41.750
";
        generic_observation_epoch_decoding_test(
            content,
            3,
            Constellation::GPS,
            &[("GPS", "C1C, L1C, D1C, S1C, C2W, L2W, D2W, S2W")],
            "2022-03-04T00:00:00 GPST",
            24,
            "2022-03-04T00:00:00 GPST",
            EpochFlag::Ok,
            None,
        );

        // let c1c = Observable::from_str("C1C").unwrap();
        // let l1c = Observable::from_str("L1C").unwrap();
        // let d1c = Observable::from_str("D1C").unwrap();
        // let s1c = Observable::from_str("S1C").unwrap();
        // let c2w = Observable::from_str("C2W").unwrap();
        // let l2w = Observable::from_str("L2W").unwrap();
        // let d2w = Observable::from_str("D2W").unwrap();
        // let s2w = Observable::from_str("S2W").unwrap();

        // let g01 = SV::from_str("G01").unwrap();
        // let g03 = SV::from_str("G03").unwrap();
        // let g04 = SV::from_str("G04").unwrap();

        // let signals = obs.signals.clone();

        // for sig in &signals {
        //     if sig.sv == g01 {
        //         if sig.observable == c1c {
        //             assert_eq!(sig.value, 20176608.780);
        //         } else if sig.observable == l1c {
        //             assert_eq!(sig.value, 106028802.118);
        //         } else if sig.observable == d1c {
        //             assert_eq!(sig.value, -1009.418);
        //         } else if sig.observable == s1c {
        //             assert_eq!(sig.value, 50.250);
        //         } else if sig.observable == c2w {
        //             assert_eq!(sig.value, 20176610.080);
        //         } else if sig.observable == l2w {
        //             assert_eq!(sig.value, 82619851.246);
        //             //assert_eq!(sig.snr, Some(SNR::from(8))); //TODO
        //         } else if sig.observable == d2w {
        //             assert_eq!(sig.value, -786.562);
        //         } else if sig.observable == s2w {
        //             assert_eq!(sig.value, 54.500);
        //         } else {
        //             panic!("found invalid observable {}", sig.observable);
        //         }
        //     } else if sig.sv == g03 {
        //         if sig.observable == c1c {
        //             assert_eq!(sig.value, 20719565.760);
        //         } else if sig.observable == l1c {
        //             assert_eq!(sig.value, 108882069.815);
        //         } else if sig.observable == d1c {
        //             assert_eq!(sig.value, 762.203);
        //         } else if sig.observable == s1c {
        //             assert_eq!(sig.value, 49.750);
        //         } else if sig.observable == c2w {
        //             assert_eq!(sig.value, 20719566.420);
        //         } else if sig.observable == l2w {
        //             assert_eq!(sig.value, 84843175.688);
        //             //assert_eq!(sig.snr, Some(SNR::from(8))); //TODO
        //         } else if sig.observable == d2w {
        //             assert_eq!(sig.value, 593.922);
        //         } else if sig.observable == s2w {
        //             assert_eq!(sig.value, 55.000);
        //         } else {
        //             panic!("found invalid observable {}", sig.observable);
        //         }
        //     } else if sig.sv == g04 {
        //         if sig.observable == c1c {
        //             assert_eq!(sig.value, 21342618.100);
        //         } else if sig.observable == l1c {
        //             assert_eq!(sig.value, 112156219.398);
        //         } else if sig.observable == d1c {
        //             assert_eq!(sig.value, 2167.688);
        //         } else if sig.observable == s1c {
        //             assert_eq!(sig.value, 48.250);
        //         } else if sig.observable == c2w {
        //             assert_eq!(sig.value, 21342617.440);
        //         } else if sig.observable == l2w {
        //             assert_eq!(sig.value, 87394474.096);
        //             //assert_eq!(sig.snr, Some(SNR::from(7))); //TODO
        //         } else if sig.observable == d2w {
        //             assert_eq!(sig.value, 1689.105);
        //         } else if sig.observable == s2w {
        //             assert_eq!(sig.value, 45.500);
        //         } else {
        //             panic!("found invalid observable {}", sig.observable);
        //         }
        //     } else {
        //         panic!("invalid sv");
        //     }
        // }
    }

    #[test]
    fn test_parse_v2_1() {
        let content = " 21 01 01 00 00 00.0000000  0 24G07G08G10G13G15G16G18G20G21G23G26G27
G30R01R02R03R08R09R15R16R17R18R19R24
  24178026.635 6  24178024.891 6                 127056391.69906  99004963.01703
                  24178026.139 3  24178024.181 3        38.066          22.286

  21866748.928 7  21866750.407 7  21866747.537 8 114910552.08207  89540700.32608
  85809828.27608  21866748.200 8  21866749.482 8        45.759          49.525
  52.161
  21458907.960 8  21458908.454 7  21458905.489 8 112767333.29708  87870655.27209
  84209365.43808  21458907.312 9  21458908.425 9        50.526          55.388
  53.157
  25107711.730 5                                 131941919.38305 102811868.09001
  25107711.069 1  25107709.586 1        33.150           8.952

  24224693.760 6  24224693.174 5                 127301651.00206  99196079.53805
                  24224693.407 5  24224691.898 5        36.121          31.645

  21749627.212 8                                 114295057.63608  89061063.16706
                  21749626.220 6  21749624.795 6        48.078          39.240

  23203962.113 6  23203960.554 6  23203963.222 7 121937655.11806  95016353.74904
  91057352.20207  23203961.787 4  23203960.356 4        41.337          28.313
";

        generic_observation_epoch_decoding_test(
            content,
            2,
            Constellation::GPS,
            &[("GPS", "C1, C2, C5, L1, L2, L5, P1, P2, S1, S2, S5")],
            "2021-01-01T00:00:00 GPST",
            62,
            "2021-01-01T00:00:00 GPST",
            EpochFlag::Ok,
            None,
        );

        // let c1 = Observable::from_str("C1").unwrap();
        // let c2 = Observable::from_str("C2").unwrap();
        // let c5 = Observable::from_str("C5").unwrap();
        // let l1 = Observable::from_str("L1").unwrap();
        // let l2 = Observable::from_str("L2").unwrap();
        // let l5 = Observable::from_str("L5").unwrap();
        // let p1 = Observable::from_str("P1").unwrap();
        // let p2 = Observable::from_str("P2").unwrap();
        // let s1 = Observable::from_str("S1").unwrap();
        // let s2 = Observable::from_str("S2").unwrap();
        // let s5 = Observable::from_str("S5").unwrap();

        // let g07 = SV::from_str("G07").unwrap();
        // let g08 = SV::from_str("G08").unwrap();
        // let g10 = SV::from_str("G10").unwrap();
        // let g13 = SV::from_str("G13").unwrap();
        // let g15 = SV::from_str("G15").unwrap();
        // let g16 = SV::from_str("G16").unwrap();
        // let g18 = SV::from_str("G18").unwrap();

        // let signals = obs.signals.clone();

        // for sig in &signals {
        //     if sig.sv == g07 {
        //         if sig.observable == c1 {
        //             assert_eq!(sig.value, 24178026.635);
        //         } else if sig.observable == c2 {
        //             assert_eq!(sig.value, 24178024.891);
        //         } else if sig.observable == c5 {
        //             panic!("found invalid obs");
        //         } else if sig.observable == l1 {
        //             assert_eq!(sig.value, 127056391.699);
        //         } else if sig.observable == l2 {
        //             assert_eq!(sig.value, 99004963.017);
        //         } else if sig.observable == l5 {
        //             panic!("found invalid obs");
        //         } else if sig.observable == p1 {
        //             assert_eq!(sig.value, 24178026.139);
        //         } else if sig.observable == p2 {
        //             assert_eq!(sig.value, 24178024.181);
        //         } else if sig.observable == s1 {
        //             assert_eq!(sig.value, 38.066);
        //         } else if sig.observable == s2 {
        //             assert_eq!(sig.value, 22.286);
        //         } else if sig.observable == s5 {
        //             panic!("found invalid obs");
        //         } else {
        //             panic!("found invalid observable {}", sig.observable);
        //         }
        //     } else if sig.sv == g08 {
        //         if sig.observable == c1 {
        //         } else if sig.observable == c2 {
        //         } else if sig.observable == c5 {
        //         } else if sig.observable == l1 {
        //         } else if sig.observable == l2 {
        //         } else if sig.observable == l5 {
        //         } else if sig.observable == p1 {
        //         } else if sig.observable == p2 {
        //         } else if sig.observable == s1 {
        //         } else if sig.observable == s2 {
        //         } else if sig.observable == s5 {
        //             assert_eq!(sig.value, 52.161);
        //         } else {
        //             panic!("found invalid observable {}", sig.observable);
        //         }
        //     } else if sig.sv == g10 {
        //     } else if sig.sv == g13 {
        //     } else if sig.sv == g15 {
        //         if sig.observable == c5 {
        //             panic!("found invalid obs");
        //         } else if sig.observable == l5 {
        //             panic!("found invalid obs");
        //         }
        //     } else if sig.sv == g16 {
        //         if sig.observable == c2 {
        //             panic!("found invalid obs");
        //         } else if sig.observable == c5 {
        //             panic!("found invalid obs");
        //         } else if sig.observable == s5 {
        //             panic!("found invalid obs");
        //         }
        //     } else if sig.sv == g18 {
        //         if sig.observable == s5 {
        //             panic!("found invalid obs");
        //         }
        //     } else {
        //         panic!("invalid sv {}", sig.sv);
        //     }
        // }
    }
}
