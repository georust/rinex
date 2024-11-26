//! RINEX header formatting

use crate::{
    fmt_comment, fmt_rinex,
    header::Header,
    prelude::{Constellation, FormattingError},
    types::Type,
};

use std::io::{BufWriter, Write};

impl Header {
    /// Formats [Header] into [Write]able interface, using efficient buffering.
    pub fn format<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        if let Some(obs) = &self.obs {
            if let Some(crinex) = &obs.crinex {
                crinex.format(w)?;
            }
        }
        self.format_rinex_version(w)?;
        self.format_prog_runby(w)?;
        self.format_observer_agency(w)?;
        self.format_comments(w)?;
        self.format_rinex_dependent(w)?;

        if let Some(position) = self.ground_position {
            writeln!(
                w,
                "{}",
                fmt_rinex(&format!("{:X}", position), "APPROX POSITION XYZ")
            )?;
        }

        self.format_sampling_interval(w)?;

        if let Some(leap) = self.leap {
            leap.format(w)?;
        }

        //TODO
        // things that could be nice to squeeze in:
        // [+] SBAS detail (detailed vehicle identity)
        // [+] RINEX 3 -> 2 observables conversion (see OBS/V2/rovn as an example)

        // Conclusion
        writeln!(w, "{}", fmt_rinex("", "END OF HEADER"))?;
        Ok(())
    }

    /// Formats "RINEX VERSION / TYPE"
    fn format_rinex_version<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        let major = self.version.major;
        let minor = self.version.minor;
        match self.rinex_type {
            Type::NavigationData => match self.constellation {
                Some(Constellation::Glonass) => {
                    writeln!(
                        w,
                        "{}",
                        fmt_rinex(
                            &format!("{:6}.{:02}           G: GLONASS NAV DATA", major, minor),
                            "RINEX VERSION / TYPE"
                        )
                    )?;
                },
                Some(c) => {
                    writeln!(
                        w,
                        "{}",
                        fmt_rinex(
                            &format!(
                                "{:6}.{:02}           NAVIGATION DATA     {:X<20}",
                                major, minor, c
                            ),
                            "RINEX VERSION / TYPE"
                        )
                    )?;
                },
                _ => panic!("constellation must be specified when formatting a NavigationData"),
            },
            Type::ObservationData => match self.constellation {
                Some(c) => {
                    writeln!(
                        w,
                        "{}",
                        fmt_rinex(
                            &format!(
                                "{:6}.{:02}           OBSERVATION DATA    {:x<20}",
                                major, minor, c
                            ),
                            "RINEX VERSION / TYPE"
                        )
                    )?;
                },
                _ => panic!("constellation must be specified when formatting ObservationData"),
            },
            Type::MeteoData => {
                writeln!(
                    w,
                    "{}",
                    fmt_rinex(
                        &format!("{:6}.{:02}           METEOROLOGICAL DATA", major, minor),
                        "RINEX VERSION / TYPE"
                    )
                )?;
            },
            Type::ClockData => {
                writeln!(
                    w,
                    "{}",
                    fmt_rinex(
                        &format!("{:6}.{:02}           CLOCK DATA", major, minor),
                        "RINEX VERSION / TYPE"
                    )
                )?;
            },
            Type::DORIS => {},
            Type::AntennaData => {},
            Type::IonosphereMaps => {},
        }

        Ok(())
    }

    /// Formats RINEX type dependent [Header] fields
    fn format_rinex_dependent<W: Write>(
        &self,
        w: &mut BufWriter<W>,
    ) -> Result<(), FormattingError> {
        if let Some(obs) = &self.obs {
            obs.format(w, self.version.major)
        } else if let Some(meteo) = &self.meteo {
            meteo.format(w)
        } else if let Some(clock) = &self.clock {
            clock.format(w)
        } else if let Some(ionex) = &self.ionex {
            ionex.format(w)
        } else if let Some(antex) = &self.antex {
            antex.format(w)
        } else if let Some(doris) = &self.doris {
            doris.format(w)
        } else {
            Ok(()) // should not happen
        }
    }

    /// Formats "PMG / RUN BY / DATE"
    fn format_prog_runby<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        // PGM / RUN BY / DATE
        writeln!(
            w,
            "{}",
            fmt_rinex(
                &format!("{:<20}{:<20}{:<20}", self.program, self.run_by, self.date),
                "PGM / RUN BY / DATE"
            )
        )?;
        Ok(())
    }

    /// Formats "OBSERVER / AGENCY"
    fn format_observer_agency<W: Write>(
        &self,
        w: &mut BufWriter<W>,
    ) -> Result<(), FormattingError> {
        writeln!(
            w,
            "{}",
            fmt_rinex(
                &format!("{:<20}{}", self.observer, self.agency),
                "OBSERVER /AGENCY"
            )
        )?;
        Ok(())
    }

    /// Formats "INTERVAL"
    fn format_sampling_interval<W: Write>(
        &self,
        w: &mut BufWriter<W>,
    ) -> Result<(), FormattingError> {
        if let Some(interval) = &self.sampling_interval {
            writeln!(
                w,
                "{}",
                fmt_rinex(&format!("{:6}", interval.to_seconds()), "INTERVAL")
            )?;
        }
        Ok(())
    }

    /// Formats all comments
    fn format_comments<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        for comment in self.comments.iter() {
            writeln!(w, "{}", fmt_comment(comment))?;
        }
        Ok(())
    }
}
