use crate::{
    doris::{DorisKey, Observations, SignalKey, SignalObservation, Station},
    epoch::parse_in_timescale as parse_epoch_in_timescale,
    observation::ClockObservation,
    prelude::{Epoch, EpochFlag, Header, ParsingError, TimeScale},
};

/// Returns true if forwarded content does match new DORIS measurement.
pub fn is_new_epoch(line: &str) -> bool {
    line.starts_with('>')
}

fn parse_new_station(
    line: &str,
    stations: &Vec<Station>,
    signals: &mut Vec<SignalObservation>,
) -> Result<(), ParsingError> {
    let line_len = line.len();

    // station number
    let station_num = line[1..]
        .trim()
        .parse::<usize>()
        .map_err(|_| ParsingError::DorisStationFormat)?;

    // identify station
    let station = stations
        .get(station_num)
        .ok_or(ParsingError::DorisStationIdentification)?
        .clone();

    // parse first signals
    let mut offset = 4;
    loop {
        let end = (offset + 14).min(line_len);

        if line_len < end + 1 {}
        if line_len < end + 2 {}

        offset += 14;
    }

    Ok(())
}

fn parse_station_continuation(
    line: &str,
    signals: &mut Vec<SignalObservation>,
) -> Result<(), ParsingError> {
    let line_len = line.len();

    // pare signals
    let mut offset = 0;
    loop {
        let end = (offset + 14).min(line_len);

        if line_len < end + 1 {}
        if line_len < end + 2 {}

        offset += 14;
    }

    Ok(())
}

/// Parse all DORIS measurements from forwarded content.
/// Needs reference to previously parsed [Header].
pub fn parse_epoch(
    header: &Header,
    content: &str,
) -> Result<(DorisKey, Observations), ParsingError> {
    const EPOCH_SIZE: usize = "YYYY MM DD HH MM SS.NNNNNNNNN  0".len();
    const CLOCK_OFFSET: usize = 26;
    const CLOCK_SIZE: usize = 14;
    const MIN_EPOCH_SIZE: usize = EPOCH_SIZE + CLOCK_SIZE + 2;

    let mut obs_idx = 0usize;
    let mut epoch = Epoch::default();
    let flag = EpochFlag::default();

    let key = DorisKey { epoch, flag };

    let null_clock = ClockObservation::default();

    let mut observations = Observations::default();

    let mut lines = content.lines();

    let mut signals = Vec::with_capacity(8);

    let doris = header
        .doris
        .as_ref()
        .ok_or(ParsingError::MissingObservableDefinition)?;

    let observables = &doris.observables;
    let stations = &doris.stations;

    // parse TAI timestamp
    let line = lines.next().ok_or(ParsingError::EmptyEpoch)?;
    let line_len = line.len();

    if line_len < MIN_EPOCH_SIZE {
        return Err(ParsingError::EpochFormat);
    }

    let epoch = parse_epoch_in_timescale(&line[2..2 + EPOCH_SIZE], TimeScale::TAI)?;

    // parse clock field
    let offset_s = line[CLOCK_OFFSET..CLOCK_OFFSET + CLOCK_SIZE]
        .trim()
        .parse::<f64>()
        .map_err(|_| ParsingError::DorisClockParsing)?;

    observations.clock = ClockObservation::with_offset_s(&null_clock, epoch, offset_s);

    // clock extrapolated ?
    observations.clock_extrapolated = false;

    if line_len > CLOCK_OFFSET + CLOCK_SIZE {
        if line[CLOCK_OFFSET + CLOCK_SIZE..].trim().eq("1") {
            observations.clock_extrapolated = true;
        }
    }

    // parse following stations
    for line in lines {
        if line.starts_with('D') {
            parse_new_station(&line[1..], stations, &mut signals)?;
        } else {
            parse_station_continuation(line, &mut signals);
        }
    }

    Ok((key, observations))
}

#[cfg(test)]
mod test {
    use super::is_new_epoch;
    use std::str::FromStr;
    #[test]
    fn new_epoch() {
        for (desc, expected) in [
            (
                "> 2024 01 01 00 00 28.999947700  0  2       -0.151364695 0 ",
                true,
            ),
            (
                "> 2023 01 01 00 00 33.999947700  0  2       -0.151364695 0 ",
                true,
            ),
            (
                "  2023 01 01 00 00 33.999947700  0  2       -0.151364695 0 ",
                false,
            ),
            (
                "  2022 01 01 00 00 33.999947700  0  2       -0.151364695 0 ",
                false,
            ),
            ("test", false),
        ] {
            assert_eq!(is_new_epoch(desc), expected);
        }
    }

    //     #[test]
    //     fn valid_epoch() {
    //         let mut header = Header::default();
    //         let mut doris = DorisHeader::default();
    //         for obs in ["L1", "L2", "C1", "C2", "W1", "W2", "F", "P", "T", "H"] {
    //             let obs = Observable::from_str(obs).unwrap();
    //             doris.observables.push(obs);
    //         }
    //         for station in [
    //             "D01  THUB THULE                         43001S005  3   0",
    //             "D02  SVBC NY-ALESUND II                 10338S004  4   0",
    //         ] {
    //             let station = Station::from_str(station).unwrap();
    //             doris.stations.push(station);
    //         }
    //         header.doris = Some(doris);

    //         let content = "> 2024 01 01 00 00 28.999947700  0  2       -0.151364695 0
    // D01  -3237877.052    -2291024.044    21903595.62311  21903633.08011      -113.100 7
    //           -98.400 7       437.801        1002.000 1       -20.000 1        82.000 1
    // D02  -2069899.788     -407871.014     4677242.25714   4677392.20614      -119.050 7
    //          -111.000 7       437.801        1007.000 0        -2.000 0        74.000 0";

    //         let ((e, flag), content) =
    //             parse_epoch(&header, content).expect("failed to parse DORIS epoch");

    //         assert_eq!(
    //             e,
    //             Epoch::from_str("2024-01-01T00:00:28.999947700 TAI").unwrap(),
    //             "parsed wrong epoch"
    //         );
    //         assert_eq!(flag, EpochFlag::Ok, "parsed wrong epoch flag");

    //         let station = Station {
    //             key: 1,
    //             gen: 3,
    //             k_factor: 0,
    //             label: "THUB".to_string(),
    //             site: "THULE".to_string(),
    //             domes: DOMES {
    //                 site: 1,
    //                 area: 430,
    //                 sequential: 5,
    //                 point: DOMESTrackingPoint::Instrument,
    //             },
    //         };
    //         let values = content
    //             .get(&station)
    //             .unwrap_or_else(|| panic!("failed to identify {:?}", station));

    //         for (observable, data) in [
    //             (
    //                 Observable::from_str("L1C").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: None,
    //                     value: -3237877.052,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("L2").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: None,
    //                     value: -2291024.044,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("C1").unwrap(),
    //                 ObservationData {
    //                     m1: Some(1),
    //                     m2: Some(1),
    //                     value: 21903595.623,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("C2").unwrap(),
    //                 ObservationData {
    //                     m1: Some(1),
    //                     m2: Some(1),
    //                     value: 21903633.080,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("W1").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(7),
    //                     value: -113.100,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("W2").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(7),
    //                     value: -98.400,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("F").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: None,
    //                     value: 437.801,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("P").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(1),
    //                     value: 1002.000,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("T").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(1),
    //                     value: -20.0,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("H").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(1),
    //                     value: 82.0,
    //                 },
    //             ),
    //         ] {
    //             let value = values
    //                 .get(&observable)
    //                 .unwrap_or_else(|| panic!("failed to identify {:?}", observable));
    //             assert_eq!(value, &data, "wrong value parsed for {:?}", observable);
    //         }

    //         let station = Station {
    //             key: 2,
    //             gen: 4,
    //             k_factor: 0,
    //             label: "SVBC".to_string(),
    //             site: "NY-ALESUND II".to_string(),
    //             domes: DOMES {
    //                 site: 38,
    //                 area: 103,
    //                 sequential: 4,
    //                 point: DOMESTrackingPoint::Instrument,
    //             },
    //         };
    //         let values = content
    //             .get(&station)
    //             .unwrap_or_else(|| panic!("failed to identify {:?}", station));

    //         for (observable, data) in [
    //             (
    //                 Observable::from_str("L1C").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: None,
    //                     value: -2069899.788,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("L2").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: None,
    //                     value: -407871.014,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("C1").unwrap(),
    //                 ObservationData {
    //                     m1: Some(1),
    //                     m2: Some(4),
    //                     value: 4677242.257,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("C2").unwrap(),
    //                 ObservationData {
    //                     m1: Some(1),
    //                     m2: Some(4),
    //                     value: 4677392.206,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("W1").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(7),
    //                     value: -119.050,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("W2").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(7),
    //                     value: -111.000,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("F").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: None,
    //                     value: 437.801,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("P").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(0),
    //                     value: 1007.000,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("T").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(0),
    //                     value: -2.000,
    //                 },
    //             ),
    //             (
    //                 Observable::from_str("H").unwrap(),
    //                 ObservationData {
    //                     m1: None,
    //                     m2: Some(0),
    //                     value: 74.0,
    //                 },
    //             ),
    //         ] {
    //             let value = values
    //                 .get(&observable)
    //                 .unwrap_or_else(|| panic!("failed to identify {:?}", observable));
    //             assert_eq!(value, &data, "wrong value parsed for {:?}", observable);
    //         }
    //     }
}
