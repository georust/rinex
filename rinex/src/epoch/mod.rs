use crate::types::Type;
use hifitime::Epoch;
use std::str::FromStr;
use thiserror::Error;

pub mod flag;
pub use flag::EpochFlag;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse epoch flag")]
    EpochFlag(#[from] flag::Error),
    #[error("failed to parse utc timestamp")]
    EpochError(#[from] hifitime::Errors),
    #[error("expecting \"yyyy mm dd hh mm ss.ssss xx\" format")]
    FormatError,
    #[error("failed to parse seconds + nanos")]
    SecsNanosError(#[from] std::num::ParseFloatError),
    #[error("failed to parse \"yyyy\" field")]
    YearError,
    #[error("failed to parse \"m\" month field")]
    MonthError,
    #[error("failed to parse \"d\" day field")]
    DayError,
    #[error("failed to parse \"hh\" field")]
    HoursError,
    #[error("failed to parse \"mm\" field")]
    MinutesError,
    #[error("failed to parse \"ss\" field")]
    SecondsError,
    #[error("failed to parse \"ns\" field")]
    NanosecsError,
}

/*
 * Infaillible `Epoch::now()` call.
 */
pub(crate) fn now() -> Epoch {
    Epoch::now().unwrap_or(Epoch::from_gregorian_utc_at_midnight(2000, 01, 01))
}

/*
 * Formats given epoch to string, matching standard specifications
 */
pub(crate) fn format(epoch: Epoch, flag: Option<EpochFlag>, t: Type, revision: u8) -> String {
    let (y, m, d, hh, mm, ss, nanos) = epoch.to_gregorian_utc();

    match t {
        Type::ObservationData => {
            if revision < 3 {
                // old RINEX wants 2 digit YY field
                let mut y = y - 2000;
                if y < 0 {
                    // fix: files recorded prior 21st century
                    y += 100;
                }
                format!(
                    "{:02} {:>2} {:>2} {:>2} {:>2} {:>2}.{:07}  {}",
                    y,
                    m,
                    d,
                    hh,
                    mm,
                    ss,
                    nanos / 100,
                    flag.unwrap_or(EpochFlag::Ok)
                )
            } else {
                format!(
                    "{:04} {:02} {:02} {:02} {:02} {:>2}.{:07}  {}",
                    y,
                    m,
                    d,
                    hh,
                    mm,
                    ss,
                    nanos / 100,
                    flag.unwrap_or(EpochFlag::Ok)
                )
            }
        },
        Type::NavigationData => {
            if revision < 3 {
                // old RINEX wants 2 digit YY field
                let mut y = y - 2000;
                if y < 0 {
                    // fix: files recorded prior 21st century
                    y += 100;
                }
                format!(
                    "{:02} {:>2} {:>2} {:>2} {:>2} {:>2}.{:1}",
                    y,
                    m,
                    d,
                    hh,
                    mm,
                    ss,
                    nanos / 100_000_000
                )
            } else {
                format!("{:04} {:02} {:02} {:02} {:02} {:02}", y, m, d, hh, mm, ss)
            }
        },
        Type::IonosphereMaps => format!(
            "{:04}   {:>2}    {:>2}    {:>2}    {:>2}    {:>2}",
            y, m, d, hh, mm, ss
        ),
        _ => {
            if revision < 3 {
                // old RINEX wants 2 digit YY field
                let mut y = y - 2000;
                if y < 0 {
                    // fix: files recorded prior 21st century
                    y += 100;
                }
                format!("{:02} {:>2} {:>2} {:>2} {:>2} {:>2}", y, m, d, hh, mm, ss)
            } else {
                format!("{:04} {:>2} {:>2} {:>2} {:>2} {:>2}", y, m, d, hh, mm, ss)
            }
        },
    }
}

/*
 * Parses an Epoch and optional flag, from standard specifications.
 * YY encoded on two digits, prior 20000 get shifted to 21st century.
 */
pub(crate) fn parse(s: &str) -> Result<(Epoch, EpochFlag), Error> {
    let items: Vec<&str> = s.split_ascii_whitespace().collect();
    if items.len() != 6 && items.len() != 7 {
        return Err(Error::FormatError);
    }
    if let Ok(mut y) = i32::from_str_radix(items[0], 10) {
        if y < 100 {
            // two digit issues (old rinex format)
            if y < 80 {
                // RINEX did not exist
                // modern file (2000+) that uses old revision,
                y += 2000;
            } else {
                y += 1900; // [1980:2000]
            }
        }
        if let Ok(m) = u8::from_str_radix(items[1], 10) {
            if let Ok(d) = u8::from_str_radix(items[2], 10) {
                if let Ok(hh) = u8::from_str_radix(items[3], 10) {
                    if let Ok(mm) = u8::from_str_radix(items[4], 10) {
                        if let Some(dot) = items[5].find(".") {
                            let is_nav = items[5].trim().len() < 7;
                            if let Ok(ss) = u8::from_str_radix(&items[5][..dot].trim(), 10) {
                                if let Ok(mut ns) =
                                    u32::from_str_radix(&items[5][dot + 1..].trim(), 10)
                                {
                                    if is_nav {
                                        // NAV RINEX:
                                        // precision is 0.1 sec
                                        ns *= 100_000_000;
                                    } else {
                                        // OBS RINEX:
                                        // precision is 0.1 usec
                                        ns *= 100;
                                    }
                                    let e = Epoch::from_gregorian_utc(y, m, d, hh, mm, ss, ns);
                                    if items.len() == 7 {
                                        // flag exists
                                        Ok((e, EpochFlag::from_str(items[6].trim())?))
                                    } else {
                                        Ok((e, EpochFlag::default()))
                                    }
                                } else {
                                    Err(Error::NanosecsError)
                                }
                            } else {
                                Err(Error::SecondsError)
                            }
                        } else {
                            /*
                             * no nanoseconds to parse,
                             * we assume no flags either. Flags only come in Observation epochs
                             * that always have nanoseconds specified */
                            if let Ok(ss) = u8::from_str_radix(&items[5].trim(), 10) {
                                let e = Epoch::from_gregorian_utc(y, m, d, hh, mm, ss, 0);
                                Ok((e, EpochFlag::Ok))
                            } else {
                                Err(Error::SecondsError)
                            }
                        }
                    } else {
                        Err(Error::MinutesError)
                    }
                } else {
                    Err(Error::HoursError)
                }
            } else {
                Err(Error::DayError)
            }
        } else {
            Err(Error::MonthError)
        }
    } else {
        Err(Error::YearError)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hifitime::TimeScale;
    #[test]
    fn epoch_parse_nav_v2() {
        let e = parse("20 12 31 23 45  0.0");
        assert_eq!(e.is_ok(), true);
        let (e, flag) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2020);
        assert_eq!(m, 12);
        assert_eq!(d, 31);
        assert_eq!(hh, 23);
        assert_eq!(mm, 45);
        assert_eq!(ss, 0);
        assert_eq!(ns, 0);
        assert_eq!(e.time_scale, TimeScale::UTC);
        assert_eq!(flag, EpochFlag::Ok);
        assert_eq!(
            format(e, None, Type::NavigationData, 2),
            "20 12 31 23 45  0.0"
        );

        let e = parse("21  1  1 16 15  0.0");
        assert_eq!(e.is_ok(), true);
        let (e, flag) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2021);
        assert_eq!(m, 1);
        assert_eq!(d, 1);
        assert_eq!(hh, 16);
        assert_eq!(mm, 15);
        assert_eq!(ss, 0);
        assert_eq!(ns, 0);
        assert_eq!(e.time_scale, TimeScale::UTC);
        assert_eq!(flag, EpochFlag::Ok);
        assert_eq!(
            format(e, None, Type::NavigationData, 2),
            "21  1  1 16 15  0.0"
        );
    }
    #[test]
    fn epoch_parse_nav_v2_nanos() {
        let e = parse("20 12 31 23 45  0.1");
        assert_eq!(e.is_ok(), true);
        let (e, _) = e.unwrap();
        let (_, _, _, _, _, ss, ns) = e.to_gregorian_utc();
        assert_eq!(ss, 0);
        assert_eq!(ns, 100_000_000);
        assert_eq!(
            format(e, None, Type::NavigationData, 2),
            "20 12 31 23 45  0.1"
        );
    }
    #[test]
    fn epoch_parse_nav_v3() {
        let e = parse("2021 01 01 00 00 00 ");
        assert_eq!(e.is_ok(), true);
        let (e, _) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2021);
        assert_eq!(m, 1);
        assert_eq!(d, 1);
        assert_eq!(hh, 00);
        assert_eq!(mm, 00);
        assert_eq!(ss, 0);
        assert_eq!(ns, 0);
        assert_eq!(e.time_scale, TimeScale::UTC);
        assert_eq!(
            format(e, None, Type::NavigationData, 3),
            "2021 01 01 00 00 00"
        );

        let e = parse("2021 01 01 09 45 00 ");
        assert_eq!(e.is_ok(), true);
        let (e, _) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2021);
        assert_eq!(m, 1);
        assert_eq!(d, 1);
        assert_eq!(hh, 09);
        assert_eq!(mm, 45);
        assert_eq!(ss, 0);
        assert_eq!(ns, 0);
        assert_eq!(
            format(e, None, Type::NavigationData, 3),
            "2021 01 01 09 45 00"
        );

        let e = parse("2020 06 25 00 00 00");
        assert_eq!(e.is_ok(), true);
        let (e, _) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2020);
        assert_eq!(m, 6);
        assert_eq!(d, 25);
        assert_eq!(hh, 00);
        assert_eq!(mm, 00);
        assert_eq!(ss, 0);
        assert_eq!(ns, 0);
        assert_eq!(
            format(e, None, Type::NavigationData, 3),
            "2020 06 25 00 00 00"
        );

        let e = parse("2020 06 25 09 49 04");
        assert_eq!(e.is_ok(), true);
        let (e, _) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2020);
        assert_eq!(m, 6);
        assert_eq!(d, 25);
        assert_eq!(hh, 09);
        assert_eq!(mm, 49);
        assert_eq!(ss, 04);
        assert_eq!(ns, 0);
        assert_eq!(
            format(e, None, Type::NavigationData, 3),
            "2020 06 25 09 49 04"
        );
    }
    #[test]
    fn epoch_parse_obs_v2() {
        let e = parse(" 21 12 21  0  0  0.0000000  0");
        assert_eq!(e.is_ok(), true);
        let (e, flag) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2021);
        assert_eq!(m, 12);
        assert_eq!(d, 21);
        assert_eq!(hh, 00);
        assert_eq!(mm, 00);
        assert_eq!(ss, 0);
        assert_eq!(ns, 0);
        assert_eq!(e.time_scale, TimeScale::UTC);
        assert_eq!(flag, EpochFlag::Ok);
        assert_eq!(
            format(e, None, Type::ObservationData, 2),
            "21 12 21  0  0  0.0000000  0"
        );

        let e = parse(" 21 12 21  0  0 30.0000000  0");
        assert_eq!(e.is_ok(), true);
        let (e, flag) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2021);
        assert_eq!(m, 12);
        assert_eq!(d, 21);
        assert_eq!(hh, 00);
        assert_eq!(mm, 00);
        assert_eq!(ss, 30);
        assert_eq!(ns, 0);
        assert_eq!(flag, EpochFlag::Ok);
        assert_eq!(
            format(e, None, Type::ObservationData, 2),
            "21 12 21  0  0 30.0000000  0"
        );

        let e = parse(" 21 12 21  0  0 30.0000000  1");
        assert_eq!(e.is_ok(), true);
        let (_e, flag) = e.unwrap();
        assert_eq!(flag, EpochFlag::PowerFailure);
        //assert_eq!(format!("{:o}", e), "21 12 21  0  0 30.0000000  1");

        let e = parse(" 21 12 21  0  0 30.0000000  2");
        assert_eq!(e.is_ok(), true);
        let (_e, flag) = e.unwrap();
        assert_eq!(flag, EpochFlag::AntennaBeingMoved);

        let e = parse(" 21 12 21  0  0 30.0000000  3");
        assert_eq!(e.is_ok(), true);
        let (_e, flag) = e.unwrap();
        assert_eq!(flag, EpochFlag::NewSiteOccupation);

        let e = parse(" 21 12 21  0  0 30.0000000  4");
        assert_eq!(e.is_ok(), true);
        let (_e, flag) = e.unwrap();
        assert_eq!(flag, EpochFlag::HeaderInformationFollows);

        let e = parse(" 21 12 21  0  0 30.0000000  5");
        assert_eq!(e.is_ok(), true);
        let (_e, flag) = e.unwrap();
        assert_eq!(flag, EpochFlag::ExternalEvent);

        let e = parse(" 21 12 21  0  0 30.0000000  6");
        assert_eq!(e.is_ok(), true);
        let (_e, flag) = e.unwrap();
        assert_eq!(flag, EpochFlag::CycleSlip);

        let e = parse(" 21  1  1  0  0  0.0000000  0");
        assert_eq!(e.is_ok(), true);
        let (e, flag) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2021);
        assert_eq!(m, 1);
        assert_eq!(d, 1);
        assert_eq!(hh, 00);
        assert_eq!(mm, 00);
        assert_eq!(ss, 0);
        assert_eq!(ns, 0);
        assert_eq!(flag, EpochFlag::Ok);
        //assert_eq!(format!("{:o}", e), "21  1  1  0  0  0.0000000  0");

        let e = parse(" 21  1  1  0  7 30.0000000  0");
        assert_eq!(e.is_ok(), true);
        let (e, flag) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2021);
        assert_eq!(m, 1);
        assert_eq!(d, 1);
        assert_eq!(hh, 00);
        assert_eq!(mm, 7);
        assert_eq!(ss, 30);
        assert_eq!(ns, 0);
        assert_eq!(flag, EpochFlag::Ok);
        //assert_eq!(format!("{:o}", e), "21  1  1  0  7 30.0000000  0");
    }
    #[test]
    fn epoch_parse_obs_v3() {
        let e = parse(" 2022 01 09 00 00  0.0000000  0");
        assert_eq!(e.is_ok(), true);
        let (e, flag) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2022);
        assert_eq!(m, 1);
        assert_eq!(d, 9);
        assert_eq!(hh, 00);
        assert_eq!(mm, 0);
        assert_eq!(ss, 00);
        assert_eq!(ns, 0);
        assert_eq!(flag, EpochFlag::Ok);
        //assert_eq!(format!("{}", e), "2022 01 09 00 00  0.0000000  0");

        let e = parse(" 2022 01 09 00 13 30.0000000  0");
        assert_eq!(e.is_ok(), true);
        let (e, flag) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2022);
        assert_eq!(m, 1);
        assert_eq!(d, 9);
        assert_eq!(hh, 00);
        assert_eq!(mm, 13);
        assert_eq!(ss, 30);
        assert_eq!(ns, 0);
        assert_eq!(flag, EpochFlag::Ok);
        //assert_eq!(format!("{}", e), "2022 01 09 00 13 30.0000000  0");

        let e = parse(" 2022 03 04 00 52 30.0000000  0");
        assert_eq!(e.is_ok(), true);
        let (e, flag) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2022);
        assert_eq!(m, 3);
        assert_eq!(d, 4);
        assert_eq!(hh, 00);
        assert_eq!(mm, 52);
        assert_eq!(ss, 30);
        assert_eq!(ns, 0);
        assert_eq!(flag, EpochFlag::Ok);
        //assert_eq!(format!("{}", e), "2022 03 04 00 52 30.0000000  0");

        let e = parse(" 2022 03 04 00 02 30.0000000  0");
        assert_eq!(e.is_ok(), true);
        let (e, flag) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2022);
        assert_eq!(m, 3);
        assert_eq!(d, 4);
        assert_eq!(hh, 00);
        assert_eq!(mm, 02);
        assert_eq!(ss, 30);
        assert_eq!(ns, 0);
        assert_eq!(flag, EpochFlag::Ok);
        //assert_eq!(format!("{}", e), "2022 03 04 00 02 30.0000000  0");
    }
    #[test]
    fn epoch_parse_obs_v2_nanos() {
        let e = parse(" 21  1  1  0  7 39.1234567  0");
        assert_eq!(e.is_ok(), true);
        let (e, _) = e.unwrap();
        let (_, _, _, _, _, ss, ns) = e.to_gregorian_utc();
        assert_eq!(ss, 39);
        assert_eq!(ns, 123_456_700);
    }
    #[test]
    fn epoch_parse_obs_v3_nanos() {
        let e = parse("2022 01 09 00 00  0.1000000  0");
        assert_eq!(e.is_ok(), true);
        let (e, _) = e.unwrap();
        let (_, _, _, _, _, ss, ns) = e.to_gregorian_utc();
        assert_eq!(ss, 0);
        assert_eq!(ns, 100_000_000);
        //assert_eq!(format!("{}", e), "2022 01 09 00 00  0.1000000  0");

        let e = parse(" 2022 01 09 00 00  0.1234000  0");
        assert_eq!(e.is_ok(), true);
        let (e, _) = e.unwrap();
        let (_, _, _, _, _, ss, ns) = e.to_gregorian_utc();
        assert_eq!(ss, 0);
        assert_eq!(ns, 123_400_000);
        //assert_eq!(format!("{}", e), "2022 01 09 00 00  0.1234000  0");

        let e = parse(" 2022 01 09 00 00  8.7654321  0");
        assert_eq!(e.is_ok(), true);
        let (e, _) = e.unwrap();
        let (_, _, _, _, _, ss, ns) = e.to_gregorian_utc();
        assert_eq!(ss, 8);
        assert_eq!(ns, 765_432_100);
        //assert_eq!(format!("{}", e), "2022 01 09 00 00  8.7654321  0");
    }
    #[test]
    fn epoch_parse_meteo_v2() {
        let e = parse(" 22  1  4  0  0  0  ");
        assert_eq!(e.is_ok(), true);
        let (e, _) = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2022);
        assert_eq!(m, 1);
        assert_eq!(d, 4);
        assert_eq!(hh, 00);
        assert_eq!(mm, 00);
        assert_eq!(ss, 00);
        assert_eq!(ns, 0);
        //assert_eq!(format!("{}", e), "2022 03 04 00 02 30.0000000  0");
    }
}
