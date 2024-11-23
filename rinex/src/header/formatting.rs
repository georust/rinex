//! RINEX header formatting

use crate::{
    fmt_comment, fmt_rinex,
    header::Header,
    prelude::{Constellation, Epoch},
    types::Type,
};

use hifitime::Unit;

impl Header {
    /// Format RINEX Version Type
    pub(crate) fn fmt_rinex_version_type(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let major = self.version.major;
        let minor = self.version.minor;
        match self.rinex_type {
            Type::NavigationData => match self.constellation {
                Some(Constellation::Glonass) => {
                    writeln!(
                        f,
                        "{}",
                        fmt_rinex(
                            &format!("{:6}.{:02}           G: GLONASS NAV DATA", major, minor),
                            "RINEX VERSION / TYPE"
                        )
                    )
                },
                Some(c) => {
                    writeln!(
                        f,
                        "{}",
                        fmt_rinex(
                            &format!(
                                "{:6}.{:02}           NAVIGATION DATA     {:X<20}",
                                major, minor, c
                            ),
                            "RINEX VERSION / TYPE"
                        )
                    )
                },
                _ => panic!("constellation must be specified when formatting a NavigationData"),
            },
            Type::ObservationData => match self.constellation {
                Some(c) => {
                    writeln!(
                        f,
                        "{}",
                        fmt_rinex(
                            &format!(
                                "{:6}.{:02}           OBSERVATION DATA    {:x<20}",
                                major, minor, c
                            ),
                            "RINEX VERSION / TYPE"
                        )
                    )
                },
                _ => panic!("constellation must be specified when formatting ObservationData"),
            },
            Type::MeteoData => {
                writeln!(
                    f,
                    "{}",
                    fmt_rinex(
                        &format!("{:6}.{:02}           METEOROLOGICAL DATA", major, minor),
                        "RINEX VERSION / TYPE"
                    )
                )
            },
            Type::ClockData => {
                writeln!(
                    f,
                    "{}",
                    fmt_rinex(
                        &format!("{:6}.{:02}           CLOCK DATA", major, minor),
                        "RINEX VERSION / TYPE"
                    )
                )
            },
            Type::DORIS => todo!("doris formatting"),
            Type::AntennaData => todo!("antex formatting"),
            Type::IonosphereMaps => todo!("ionex formatting"),
        }
    }
    /*
     * Format rinex type dependent stuff
     */
    pub(crate) fn fmt_rinex_dependent(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.rinex_type {
            Type::ObservationData => self.fmt_observation_rinex(f),
            Type::MeteoData => self.fmt_meteo_rinex(f),
            Type::NavigationData => Ok(()),
            Type::ClockData => self.fmt_clock_rinex(f),
            Type::IonosphereMaps => self.fmt_ionex(f),
            Type::AntennaData => Ok(()), // FIXME
            Type::DORIS => Ok(()),       // FIXME
        }
    }
    /*
     * Clock Data fields formatting
     */
    fn fmt_clock_rinex(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(clock) = &self.clock {
            // Types of data: observables equivalent
            let mut descriptor = String::new();
            descriptor.push_str(&format!("{:6}", clock.codes.len()));
            for (i, observable) in clock.codes.iter().enumerate() {
                if (i % 9) == 0 && i > 0 {
                    descriptor.push_str("      "); // TAB
                }
                descriptor.push_str(&format!("{:6}", observable));
            }
            writeln!(f, "{}", fmt_rinex(&descriptor, "# / TYPES OF DATA"))?;

            // possible timescale
            if let Some(ts) = clock.timescale {
                writeln!(
                    f,
                    "{}",
                    fmt_rinex(&format!("   {:x}", ts), "TIME SYSTEM ID")
                )?;
            }
            // TODO: missing fields
            //if let Some(agency) = &clock.agency {
            //    writeln!(
            //        f,
            //        "{}",
            //        fmt_rinex(
            //            &format!("{:<5} {}", agency.code, agency.name),
            //            "ANALYSIS CENTER"
            //        )
            //    )?;
            //}
        }
        Ok(())
    }
    /*
     * IONEX fields formatting
     */
    fn fmt_ionex(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(ionex) = &self.ionex {
            writeln!(
                f,
                "{}",
                fmt_rinex(&format!("{:6}", ionex.map_dimension), "MAP DIMENSION")
            )?;
            // h grid
            let (start, end, spacing) = (
                ionex.grid.height.start,
                ionex.grid.height.end,
                ionex.grid.height.spacing,
            );
            writeln!(
                f,
                "{}",
                fmt_rinex(
                    &format!("{} {} {}", start, end, spacing),
                    "HGT1 / HGT2 / DHGT"
                )
            )?;
            // lat grid
            let (start, end, spacing) = (
                ionex.grid.latitude.start,
                ionex.grid.latitude.end,
                ionex.grid.latitude.spacing,
            );
            writeln!(
                f,
                "{}",
                fmt_rinex(
                    &format!("{} {} {}", start, end, spacing),
                    "LAT1 / LAT2 / DLAT"
                )
            )?;
            // lon grid
            let (start, end, spacing) = (
                ionex.grid.longitude.start,
                ionex.grid.longitude.end,
                ionex.grid.longitude.spacing,
            );
            writeln!(
                f,
                "{}",
                fmt_rinex(
                    &format!("{} {} {}", start, end, spacing),
                    "LON1 / LON2 / DLON"
                )
            )?;
            // elevation cutoff
            writeln!(
                f,
                "{}",
                fmt_rinex(&format!("{}", ionex.elevation_cutoff), "ELEVATION CUTOFF")
            )?;
            // mapping func
            if let Some(func) = &ionex.mapping {
                writeln!(
                    f,
                    "{}",
                    fmt_rinex(&format!("{:?}", func), "MAPPING FUNCTION")
                )?;
            } else {
                writeln!(f, "{}", fmt_rinex("NONE", "MAPPING FUNCTION"))?;
            }
            // time of first map
            writeln!(f, "{}", fmt_rinex("TODO", "EPOCH OF FIRST MAP"))?;
            // time of last map
            writeln!(f, "{}", fmt_rinex("TODO", "EPOCH OF LAST MAP"))?;
        }
        Ok(())
    }
    /*
     * Meteo Data fields formatting
     */
    fn fmt_meteo_rinex(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(meteo) = &self.meteo {
            /*
             * List of observables
             */
            let mut descriptor = String::new();
            descriptor.push_str(&format!("{:6}", meteo.codes.len()));
            for (i, observable) in meteo.codes.iter().enumerate() {
                if (i % 9) == 0 && i > 0 {
                    descriptor.push_str("      "); // TAB
                }
                descriptor.push_str(&format!("    {}", observable));
            }
            writeln!(f, "{}", fmt_rinex(&descriptor, "# / TYPES OF OBSERV"))?;
            for sensor in &meteo.sensors {
                write!(f, "{}", sensor)?;
            }
        }
        Ok(())
    }
    /*
     * Observation Data fields formatting
     */
    fn fmt_observation_rinex(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(obs) = &self.obs {
            if let Some(e) = obs.timeof_first_obs {
                let (y, m, d, hh, mm, ss, nanos) =
                    (e + e.leap_seconds(true).unwrap_or(0.0) * Unit::Second).to_gregorian_utc();
                writeln!(
                    f,
                    "{}",
                    fmt_rinex(
                        &format!(
                            "  {:04}    {:02}    {:02}    {:02}    {:02}   {:02}.{:07}     {:x}",
                            y, m, d, hh, mm, ss, nanos, e.time_scale
                        ),
                        "TIME OF FIRST OBS"
                    )
                )?;
            }
            if let Some(e) = obs.timeof_last_obs {
                let (y, m, d, hh, mm, ss, nanos) =
                    (e + e.leap_seconds(true).unwrap_or(0.0) * Unit::Second).to_gregorian_utc();
                writeln!(
                    f,
                    "{}",
                    fmt_rinex(
                        &format!(
                            "  {:04}    {:02}    {:02}    {:02}    {:02}   {:02}.{:07}     {:x}",
                            y, m, d, hh, mm, ss, nanos, e.time_scale
                        ),
                        "TIME OF LAST OBS"
                    )
                )?;
            }
            /*
             * Form the observables list
             */
            match self.version.major {
                1 | 2 => {
                    /*
                     * List of observables
                     */
                    let mut descriptor = String::new();
                    if let Some((_constell, observables)) = obs.codes.iter().next() {
                        descriptor.push_str(&format!("{:6}", observables.len()));
                        for (i, observable) in observables.iter().enumerate() {
                            if (i % 9) == 0 && i > 0 {
                                descriptor.push_str("      "); // TAB
                            }
                            descriptor.push_str(&format!("    {}", observable));
                        }
                        writeln!(f, "{}", fmt_rinex(&descriptor, "# / TYPES OF OBSERV"))?;
                    }
                },
                _ => {
                    /*
                     * List of observables
                     */
                    for (constell, observables) in &obs.codes {
                        let mut descriptor = String::new();
                        descriptor.push_str(&format!("{:x}{:5}", constell, observables.len()));
                        for (i, observable) in observables.iter().enumerate() {
                            if (i % 13) == 0 && (i > 0) {
                                descriptor.push_str("        "); // TAB
                            }
                            descriptor.push_str(&format!(" {}", observable)); // TAB
                        }
                        writeln!(f, "{}", fmt_rinex(&descriptor, "SYS / # / OBS TYPES"))?;
                    }
                },
            }
            // must take place after list of observables:
            //  TODO DCBS compensations
            //  TODO PCVs compensations
        }
        Ok(())
    }
    /*
     * Format all comments
     */
    pub(crate) fn fmt_comments(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for comment in self.comments.iter() {
            writeln!(f, "{}", fmt_comment(comment))?;
        }
        Ok(())
    }
}
