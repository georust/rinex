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
///   - key: previously identified [ObsKey]
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
                signals.push(SignalObservation {
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
                println!("{} obs {} [{}]", sv, value, i);
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
            Constellation, Epoch, Header, Observable, Observations, TimeScale, Version, SNR, SV,
        },
        tests::toolkit::{observables_csv as observables_from_csv, sv_csv as sv_from_csv},
    };
    use itertools::Itertools;
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
        let mut obs = Observations::default();

        let t0 = Epoch::from_str("2022-03-04T00:00:00 GPST").unwrap();

        let mut specs = SpecFields::default().with_time_of_first_obs(t0);

        specs.codes.insert(
            Constellation::GPS,
            "C1C, L1C, S1C, C2P, C2W, C2S, C2L, C2X, L2P, L2W, L2S, L2L, L2X, S2P, S2W, S2S, S2L, S2X"
                .split(',')
                .map(|s| Observable::from_str(s).unwrap())
                .collect::<Vec<_>>(),
        );

        let header = &Header::default()
            .with_constellation(Constellation::GPS)
            .with_observation_fields(specs);

        let ts = TimeScale::GPST;

        let content =
"> 2022 03 04  0  0  0.0000000  0 22        .000000000000
G01  20832393.682   109474991.854 8        49.500                    20832389.822                                                                    85305196.437 8                                                                        49.500
G03  20342516.786   106900663.487 8        50.000                    20342512.006                                                                    83299201.382 8                                                                        50.000
G04  22448754.952   117969025.322 8        48.250                    22448749.312                                                                    91923884.833 7                                                                        43.750
G06  24827263.216   130468159.526 6        39.750                    24827259.316                                                                   101663482.505 6                                                                        37.250
G09  25493930.890   133971510.403 6        41.250                    25493926.950                                                                   104393373.997 6                                                                        41.750
";
        let key = parse_epoch(header, content, ts, &mut obs).unwrap();

        assert_eq!(key.epoch, t0);
        assert!(key.flag.is_ok());
        assert!(obs.clock.is_none());

        let c1c = Observable::from_str("C1C").unwrap();
        let l1c = Observable::from_str("L1C").unwrap();
        let s1c = Observable::from_str("S1C").unwrap();
        let c2w = Observable::from_str("C2W").unwrap();
        let l2w = Observable::from_str("L2W").unwrap();
        let s2w = Observable::from_str("S2W").unwrap();

        let g01 = SV::from_str("G01").unwrap();
        let g03 = SV::from_str("G03").unwrap();
        let g04 = SV::from_str("G04").unwrap();
        let g06 = SV::from_str("G06").unwrap();
        let g09 = SV::from_str("G09").unwrap();

        let signals = obs.signals.clone();

        for sig in &signals {
            if sig.sv == g01 {
                if sig.observable == c1c {
                    assert_eq!(sig.value, 20832393.682);
                } else if sig.observable == l1c {
                    assert_eq!(sig.value, 109474991.854);
                } else if sig.observable == s1c {
                    assert_eq!(sig.value, 49.500);
                } else if sig.observable == c2w {
                    assert_eq!(sig.value, 20832389.822);
                } else if sig.observable == l2w {
                    assert_eq!(sig.value, 85305196.437);
                    //assert_eq!(sig.snr, Some(SNR::from(8))); //TODO
                } else if sig.observable == s2w {
                    assert_eq!(sig.value, 49.500);
                } else {
                    panic!("found invalid observable {}", sig.observable);
                }
            } else if sig.sv == g03 {
                if sig.observable == c1c {
                    assert_eq!(sig.value, 20342516.786);
                } else if sig.observable == l1c {
                    assert_eq!(sig.value, 106900663.487);
                } else if sig.observable == s1c {
                    assert_eq!(sig.value, 50.0);
                } else if sig.observable == c2w {
                    assert_eq!(sig.value, 20342512.006);
                } else if sig.observable == l2w {
                    assert_eq!(sig.value, 83299201.382);
                    //assert_eq!(sig.snr, Some(SNR::from(8))); //TODO
                } else if sig.observable == s2w {
                    assert_eq!(sig.value, 50.000);
                } else {
                    panic!("found invalid observable {}", sig.observable);
                }
            } else if sig.sv == g04 {
                if sig.observable == c1c {
                    assert_eq!(sig.value, 22448754.952);
                } else if sig.observable == l1c {
                    assert_eq!(sig.value, 117969025.322);
                } else if sig.observable == s1c {
                    assert_eq!(sig.value, 48.250);
                } else if sig.observable == c2w {
                    assert_eq!(sig.value, 22448749.312);
                } else if sig.observable == l2w {
                    assert_eq!(sig.value, 91923884.833);
                    //assert_eq!(sig.snr, Some(SNR::from(7))); //TODO
                } else if sig.observable == s2w {
                    assert_eq!(sig.value, 43.750);
                } else {
                    panic!("found invalid observable {}", sig.observable);
                }
            } else if sig.sv == g06 {
                if sig.observable == c1c {
                    assert_eq!(sig.value, 24827263.216);
                } else if sig.observable == l1c {
                    assert_eq!(sig.value, 130468159.526);
                } else if sig.observable == s1c {
                    assert_eq!(sig.value, 39.750);
                } else if sig.observable == c2w {
                    assert_eq!(sig.value, 24827259.316);
                } else if sig.observable == l2w {
                    assert_eq!(sig.value, 101663482.505);
                    //assert_eq!(sig.snr, Some(SNR::from(6))); //TODO
                } else if sig.observable == s2w {
                    assert_eq!(sig.value, 37.250);
                } else {
                    panic!("found invalid observable {}", sig.observable);
                }
            } else if sig.sv == g09 {
                if sig.observable == c1c {
                    assert_eq!(sig.value, 25493930.890);
                } else if sig.observable == l1c {
                    assert_eq!(sig.value, 133971510.403);
                } else if sig.observable == s1c {
                    assert_eq!(sig.value, 41.250);
                } else if sig.observable == c2w {
                    assert_eq!(sig.value, 25493926.950);
                } else if sig.observable == l2w {
                    assert_eq!(sig.value, 104393373.997);
                    //assert_eq!(sig.snr, Some(SNR::from(6))); //TODO
                } else if sig.observable == s2w {
                    assert_eq!(sig.value, 41.75);
                } else {
                    panic!("found invalid observable {}", sig.observable);
                }
            } else {
                panic!("invalid sv");
            }
        }

        assert_eq!(obs.signals.len(), 30);

        let signals = obs.signals.clone();

        let formatted = fmt_observations(3, &header, &key, &None, signals);

        // TODO assert_eq!(formatted, content); // reciprocal
    }

    #[test]
    fn test_parse_v3_2() {
        let mut obs = Observations::default();

        let t0 = Epoch::from_str("2022-03-04T00:00:00 GPST").unwrap();

        let mut specs = SpecFields::default().with_time_of_first_obs(t0);

        specs.codes.insert(
            Constellation::GPS,
            "C1C, L1C, D1C, S1C, C2W, L2W, D2W, S2W"
                .split(',')
                .map(|s| Observable::from_str(s).unwrap())
                .collect::<Vec<_>>(),
        );

        let header = &Header::default()
            .with_constellation(Constellation::GPS)
            .with_observation_fields(specs);

        let ts = TimeScale::GPST;

        let content =
"> 2022 03 04 00 00  0.0000000  0  9
G01  20176608.780   106028802.11808     -1009.418          50.250    20176610.080    82619851.24609      -786.562          54.500
G03  20719565.760   108882069.81508       762.203          49.750    20719566.420    84843175.68809       593.922          55.000
G04  21342618.100   112156219.39808      2167.688          48.250    21342617.440    87394474.09607      1689.105          45.500                                                                 104393373.997 6                                                                        41.750
";
        let key = parse_epoch(header, content, ts, &mut obs).unwrap();

        assert_eq!(key.epoch, t0);
        assert!(key.flag.is_ok());
        assert!(obs.clock.is_none());

        let c1c = Observable::from_str("C1C").unwrap();
        let l1c = Observable::from_str("L1C").unwrap();
        let d1c = Observable::from_str("D1C").unwrap();
        let s1c = Observable::from_str("S1C").unwrap();
        let c2w = Observable::from_str("C2W").unwrap();
        let l2w = Observable::from_str("L2W").unwrap();
        let d2w = Observable::from_str("D2W").unwrap();
        let s2w = Observable::from_str("S2W").unwrap();

        let g01 = SV::from_str("G01").unwrap();
        let g03 = SV::from_str("G03").unwrap();
        let g04 = SV::from_str("G04").unwrap();

        let signals = obs.signals.clone();

        for sig in &signals {
            if sig.sv == g01 {
                if sig.observable == c1c {
                    assert_eq!(sig.value, 20176608.780);
                } else if sig.observable == l1c {
                    assert_eq!(sig.value, 106028802.118);
                } else if sig.observable == d1c {
                    assert_eq!(sig.value, -1009.418);
                } else if sig.observable == s1c {
                    assert_eq!(sig.value, 50.250);
                } else if sig.observable == c2w {
                    assert_eq!(sig.value, 20176610.080);
                } else if sig.observable == l2w {
                    assert_eq!(sig.value, 82619851.246);
                    //assert_eq!(sig.snr, Some(SNR::from(8))); //TODO
                } else if sig.observable == d2w {
                    assert_eq!(sig.value, -786.562);
                } else if sig.observable == s2w {
                    assert_eq!(sig.value, 54.500);
                } else {
                    panic!("found invalid observable {}", sig.observable);
                }
            } else if sig.sv == g03 {
                if sig.observable == c1c {
                    assert_eq!(sig.value, 20719565.760);
                } else if sig.observable == l1c {
                    assert_eq!(sig.value, 108882069.815);
                } else if sig.observable == d1c {
                    assert_eq!(sig.value, 762.203);
                } else if sig.observable == s1c {
                    assert_eq!(sig.value, 49.750);
                } else if sig.observable == c2w {
                    assert_eq!(sig.value, 20719566.420);
                } else if sig.observable == l2w {
                    assert_eq!(sig.value, 84843175.688);
                    //assert_eq!(sig.snr, Some(SNR::from(8))); //TODO
                } else if sig.observable == d2w {
                    assert_eq!(sig.value, 593.922);
                } else if sig.observable == s2w {
                    assert_eq!(sig.value, 55.000);
                } else {
                    panic!("found invalid observable {}", sig.observable);
                }
            } else if sig.sv == g04 {
                if sig.observable == c1c {
                    assert_eq!(sig.value, 21342618.100);
                } else if sig.observable == l1c {
                    assert_eq!(sig.value, 112156219.398);
                } else if sig.observable == d1c {
                    assert_eq!(sig.value, 2167.688);
                } else if sig.observable == s1c {
                    assert_eq!(sig.value, 48.250);
                } else if sig.observable == c2w {
                    assert_eq!(sig.value, 21342617.440);
                } else if sig.observable == l2w {
                    assert_eq!(sig.value, 87394474.096);
                    //assert_eq!(sig.snr, Some(SNR::from(7))); //TODO
                } else if sig.observable == d2w {
                    assert_eq!(sig.value, 1689.105);
                } else if sig.observable == s2w {
                    assert_eq!(sig.value, 45.500);
                } else {
                    panic!("found invalid observable {}", sig.observable);
                }
            } else {
                panic!("invalid sv");
            }
        }

        assert_eq!(obs.signals.len(), 24);

        let signals = obs.signals.clone();

        let formatted = fmt_observations(3, &header, &key, &None, signals);

        // TODO assert_eq!(formatted, content); // reciprocal
    }
}
