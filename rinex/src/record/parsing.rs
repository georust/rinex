use crate::{
    antex::{
        record::{is_new_epoch as is_new_antex_epoch, parse_antenna as parse_antex_antenna},
        Record as AntexRecord,
    },
    clock::{
        record::{is_new_epoch as is_new_clock_epoch, parse_epoch as parse_clock_epoch},
        ClockKey, ClockProfile, Record as ClockRecord,
    },
    doris::{
        record::{is_new_epoch as is_new_doris_epoch, parse_epoch as parse_doris_epoch},
        Record as DorisRecord,
    },
    hatanaka::DecompressorExpert,
    ionex::{
        record::{is_new_rms_plane, is_new_tec_plane, parse_plane as parse_ionex_plane},
        Record as IonexRecord,
    },
    is_rinex_comment,
    meteo::{
        record::{is_new_epoch as is_new_meteo_epoch, parse_epoch as parse_meteo_epoch},
        Record as MeteoRecord,
    },
    navigation::{
        record::{is_new_epoch as is_new_nav_epoch, parse_epoch as parse_nav_epoch},
        Record as NavRecord,
    },
    observation::{
        is_new_epoch as is_new_observation_epoch, parse_epoch as parse_observation_epoch,
        Record as ObservationRecord,
    },
    prelude::{Constellation, Epoch, Header, Observations, ParsingError, TimeScale},
    record::{Comments, Record},
    types::Type,
};

use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Read},
    str::from_utf8,
};

#[cfg(feature = "log")]
use log::error;

impl Record {
    /// Parses [Record] section by consuming [Reader] entirely.
    /// [Header] parsed by consuming [Reader] until this point is required.
    pub fn parse<R: Read>(
        header: &mut Header,
        reader: &mut BufReader<R>,
    ) -> Result<(Self, Comments), ParsingError> {
        // buffer pointers
        let mut rd_offset = 0; // total we have processed
        let mut total: usize = 0; // total bytes we have buffered
        let mut total_lines = 0; // total lines we have buffered

        // string buffer
        let mut content = String::with_capacity(1024);

        // comments management
        let mut comments: Comments = Comments::new();
        let mut comment_ts = Epoch::default();
        let mut comment_content = Vec::<String>::with_capacity(4);

        // ANTEX
        let mut atx_rec = AntexRecord::new();

        // NAV
        let mut nav_rec = NavRecord::new();

        // OBS
        let mut obs_rec = ObservationRecord::new();
        let mut observations = Observations::default();

        // CRINEX case
        let mut buf = [0; 1024];
        let mut is_crinex = false;
        let mut crinex_v3 = false;
        let mut crinex_content = String::with_capacity(1024);
        let mut gnss_observables = Default::default();

        if let Some(obs) = &header.obs {
            if let Some(crinex) = &obs.crinex {
                is_crinex = true;
                crinex_v3 = crinex.version.major > 2;
            }
            gnss_observables = obs.codes.clone();
        }

        // Build a decompressor, to be deployed based on already recovered [Header].
        // These parameters are compatible with historical RNX2CRX tool.
        let mut decompressor = DecompressorExpert::<5>::new(
            crinex_v3,
            header.constellation.unwrap_or_default(),
            gnss_observables,
        );

        // MET
        let mut met_rec = MeteoRecord::new();

        // CLK
        let mut clk_rec = ClockRecord::new();

        // DORIS
        let mut dor_rec = DorisRecord::new();

        // OBSERVATION case: timescale is either defined by
        //    [+] TIME OF FIRST header field
        //    [+] TIME OF LAST header field (flexibility, actually invalid according to specs)
        //    [+] GNSS system in case of single GNSS old RINEX
        let mut obs_ts = TimeScale::default();

        if let Some(obs) = &header.obs {
            match header.constellation {
                Some(Constellation::Mixed) | None => {
                    if let Some(t) = obs.timeof_first_obs {
                        obs_ts = t.time_scale;
                    } else {
                        if let Some(t) = obs.timeof_first_obs {
                            obs_ts = t.time_scale;
                        } else {
                            let t = obs
                                .timeof_last_obs
                                .ok_or(ParsingError::BadObsBadTimescaleDefinition)?;
                            obs_ts = t.time_scale;
                        }
                    }
                },
                Some(constellation) => {
                    obs_ts = constellation
                        .timescale()
                        .ok_or(ParsingError::BadObsBadTimescaleDefinition)?;
                },
            }
        }

        // Clock RINEX TimeScale definition.
        //   Modern revisions define it in header directly.
        //   Old revisions are once again badly defined and most likely not thought out.
        //   We default to GPST to "match" the case where this file is multi constellation
        //      and it seems that clocks steered to GPST is the most common case.
        //      For example NASA/CDDIS.com
        //   In mono constellation, we adapt to that timescale.
        let mut clk_ts = TimeScale::GPST;
        if let Some(clk) = &header.clock {
            if let Some(ts) = clk.timescale {
                clk_ts = ts;
            } else {
                if let Some(constellation) = &header.constellation {
                    if let Some(ts) = constellation.timescale() {
                        clk_ts = ts;
                    }
                }
            }
        }

        // IONEX case
        //  Default map type is TEC, it will come with identified Epoch
        //  but others may exist:
        //    in this case we used the previously identified Epoch
        //    and attach other kinds of maps
        let mut ionx_rec = IonexRecord::new();
        let mut ionex_rms_plane = false;

        // Iterate and consume, one line at a time
        while let Ok(size) = reader.read_line(&mut content) {
            if size == 0 {
                // reached EOS
                break;
            }

            total += size;
            total_lines += 1; // new line

            println!(
                " ======= CONTENT \n {} ====== [total={}/lines={},rd={}] ",
                &content[..total],
                total,
                total_lines,
                rd_offset
            );

            // (special case) COMMENTS: store as is
            if is_rinex_comment(&content[rd_offset..]) {
                let comment = content[rd_offset..].split_at(60).0.trim_end();
                comment_content.push(comment.to_string());
            }

            // (special case) IONEX exponent scaling:
            // keep up to date, so data interpretation remains correct
            if content[rd_offset..].contains("EXPONENT") {
                if let Some(ionex) = header.ionex.as_mut() {
                    let content = content[rd_offset..].split_at(60).0;
                    if let Ok(e) = content.trim().parse::<i8>() {
                        *ionex = ionex.with_exponent(e); // scaling update
                    }
                }
            }

            // // CRINEX special case:
            // //  apply decompression algorithm prior moving forward
            // let content = if is_crinex {
            //     let line_len = line.len();

            //     let size = decompressor
            //         .decompress(&line, line_len, &mut buf, 1024)
            //         .map_err(|e| ParsingError::CRINEX(e))?;

            //     let recovered = from_utf8(&buf[..size]).map_err(|_| ParsingError::BadUtf8Crinex)?;

            //     // TODO: to improve overall performance,
            //     // we should rework the buffering in this function.
            //     // Basically 'epoch_content' and Self::is_new_epoch interaction.
            //     // We should wait for a new epoch to appear and process everything prior that instant.
            //     // Currently, in the CRINEX case, this requires a new allocation just to append a single \n
            //     crinex_content.clear();
            //     crinex_content.push_str(recovered);
            //     &crinex_content
            // } else {
            //     &line
            // };

            let mut new_epoch = false;

            // total > 1 is here to make sure we have buffered one epoch
            // and new one is starting to appear
            if total_lines > 1 {
                new_epoch = Self::is_new_epoch(&content[rd_offset..], &header);

                if new_epoch {
                    // new epoch appearing: process what we have buffered
                    // parsing method is format dependent
                    println!("***MATCH*** \"{}\"\n", &content[rd_offset..]);

                    match &header.rinex_type {
                        Type::NavigationData => {
                            let constellation = &header.constellation.unwrap();
                            if let Ok((e, fr)) = parse_nav_epoch(
                                header.version,
                                *constellation,
                                &content[..rd_offset],
                            ) {
                                nav_rec
                                    .entry(e)
                                    .and_modify(|frames| frames.push(fr.clone()))
                                    .or_insert_with(|| vec![fr.clone()]);
                                comment_ts = e; // for comments storage
                            }
                        },
                        Type::ObservationData => {
                            match parse_observation_epoch(
                                header,
                                &content[..rd_offset],
                                obs_ts,
                                &mut observations,
                            ) {
                                Ok(key) => {
                                    obs_rec.insert(key, observations.clone());
                                    observations.signals.clear(); // reset for next parsing (single alloc)
                                    comment_ts = key.epoch; // for comments storage
                                },
                                #[cfg(feature = "log")]
                                Err(e) => {
                                    println!("parsing: {}", e);
                                },
                                #[cfg(not(feature = "log"))]
                                Err(e) => {
                                    println!("parsing: {}", e);
                                },
                            }
                        },
                        Type::DORIS => {
                            if let Ok((e, map)) = parse_doris_epoch(header, &content[..rd_offset]) {
                                dor_rec.insert(e, map);
                            }
                        },
                        Type::MeteoData => {
                            if let Ok((e, map)) = parse_meteo_epoch(header, &content[..rd_offset]) {
                                met_rec.insert(e, map);
                                comment_ts = e; // for comments classification & management
                            }
                        },
                        Type::ClockData => {
                            if let Ok((epoch, key, profile)) =
                                parse_clock_epoch(header.version, &content[..rd_offset], clk_ts)
                            {
                                if let Some(e) = clk_rec.get_mut(&epoch) {
                                    e.insert(key, profile);
                                } else {
                                    let mut inner: BTreeMap<ClockKey, ClockProfile> =
                                        BTreeMap::new();
                                    inner.insert(key, profile);
                                    clk_rec.insert(epoch, inner);
                                }
                                comment_ts = epoch; // for comments classification & management
                            }
                        },
                        Type::AntennaData => {
                            let (antenna, content) =
                                parse_antex_antenna(&content[..rd_offset]).unwrap();
                            atx_rec.push((antenna, content));
                        },
                        Type::IonosphereMaps => {
                            if let Ok((epoch, altitude, plane)) =
                                parse_ionex_plane(&content[..rd_offset], header, ionex_rms_plane)
                            {
                                if ionex_rms_plane {
                                    if let Some(rec_plane) = ionx_rec.get_mut(&(epoch, altitude)) {
                                        // provide RMS value for the entire plane
                                        for ((_, rec_tec), (_, tec)) in
                                            rec_plane.iter_mut().zip(plane.iter())
                                        {
                                            rec_tec.rms = tec.rms;
                                        }
                                    } else {
                                        // insert RMS values
                                        ionx_rec.insert((epoch, altitude), plane);
                                    }
                                } else if let Some(rec_plane) = ionx_rec.get_mut(&(epoch, altitude))
                                {
                                    // provide TEC value for the entire plane
                                    for ((_, rec_tec), (_, tec)) in
                                        rec_plane.iter_mut().zip(plane.iter())
                                    {
                                        rec_tec.tec = tec.tec;
                                    }
                                } else {
                                    // insert TEC values
                                    ionx_rec.insert((epoch, altitude), plane);
                                }
                            }
                        },
                    }

                    // preserve only new content for next time
                    unsafe {
                        let bytes = content.as_bytes_mut();
                        bytes.copy_within(rd_offset..rd_offset + size - 1, 0);
                    }
                }
            }

            if new_epoch {
                total -= rd_offset; // we have trashed all previous content
                total_lines = 1; // we're only left with one line
                rd_offset = size - 1; // we have processed it already
            } else {
                rd_offset += size; // we have processed everything
            }
        } //loop

        // wrap content and exit
        let record = match &header.rinex_type {
            Type::AntennaData => Record::AntexRecord(atx_rec),
            Type::ClockData => Record::ClockRecord(clk_rec),
            Type::IonosphereMaps => Record::IonexRecord(ionx_rec),
            Type::MeteoData => Record::MeteoRecord(met_rec),
            Type::NavigationData => Record::NavRecord(nav_rec),
            Type::ObservationData => {
                println!("debug obs_rec={:#?}", obs_rec);
                Record::ObsRecord(obs_rec)
            },
            Type::DORIS => Record::DorisRecord(dor_rec),
        };
        Ok((record, comments))
    }

    fn is_new_epoch(line: &str, header: &Header) -> bool {
        if is_rinex_comment(line) {
            return false;
        }
        match &header.rinex_type {
            Type::AntennaData => is_new_antex_epoch(line),
            Type::ClockData => is_new_clock_epoch(line),
            Type::IonosphereMaps => is_new_tec_plane(line) || is_new_rms_plane(line),
            Type::NavigationData => is_new_nav_epoch(line, header.version),
            Type::ObservationData => is_new_observation_epoch(line, header.version),
            Type::MeteoData => is_new_meteo_epoch(line, header.version),
            Type::DORIS => is_new_doris_epoch(line),
        }
    }
}
