//! Streaming contracts: independent source columns, event state and failure boundaries.
use super::{epoch, observation_rinex, source_observations, synthetic_obs};
use crate::{observation::*, prelude::*};
use std::{
    cell::Cell,
    io::{BufRead, BufReader, Cursor, Read},
    rc::Rc,
    str::FromStr,
};

fn collect(text: &str) -> ObsCollection {
    ObservationReader::new(Cursor::new(text.as_bytes()))
        .unwrap()
        .collect_records(ObsDuplicatePolicy::Preserve)
        .unwrap()
}

fn event(flag: u8, records: &[String]) -> String {
    // Synthetic Table A3 event with deliberately absent timestamp.
    format!(
        ">{:28}{flag:>3}{:>3}\n{}",
        "",
        records.len(),
        records.iter().map(|r| format!("{r}\n")).collect::<String>()
    )
}
fn record(content: &str, label: &str) -> String {
    format!("{content:<60}{label}")
}

#[test]
fn real_source_columns_owned_results_and_complete_collection_agree() {
    let text = source_observations();
    let mut reader = ObservationReader::new(Cursor::new(text.as_bytes())).unwrap();
    assert_eq!(reader.status(), ReadStatus::Reading);
    let a = reader.next().unwrap().unwrap();
    let b = reader.next().unwrap().unwrap();
    assert_eq!(a.source.block_index, Some(0));
    assert_eq!(b.source.block_index, Some(1));
    assert_eq!(a.source.body_lines, 1..3);
    assert_eq!(a.source.bytes.start, text.find('>').unwrap() as u64);
    assert_eq!(a.source.bytes.end, b.source.bytes.start);
    let ObsOutcome::Observation { key, data, .. } = &a.outcome else {
        panic!("{a:?}")
    };
    assert_eq!(key.epoch.to_string(), "2024-05-10T00:00:00 GPST");
    assert_eq!(data.signals.len(), 9);
    assert_eq!(data.signals[0].sv, SV::from_str("G03").unwrap());
    assert_eq!(data.signals[0].value, 20_888_140.504);
    let ObsOutcome::Observation { data: second, .. } = &b.outcome else {
        panic!("{b:?}")
    };
    assert_eq!(second.signals.len(), 6);
    assert_eq!(second.signals[0].sv, SV::from_str("G04").unwrap());
    assert!(reader.next().is_none());
    assert_eq!(reader.status(), ReadStatus::EndOfInput);
    assert!(reader.next().is_none());
    // Advancing or dropping the reader cannot change an earlier owned result.
    drop(reader);
    assert_eq!(data.signals[0].value, 20_888_140.504);
    let all = collect(&text);
    assert_eq!(all.items.len(), 2);
    assert_eq!(all.status, ReadStatus::EndOfInput);
    let native = super::parse(&text).unwrap();
    for item in all.items {
        let ObsOutcome::Observation { key, data, .. } = item.outcome else {
            panic!()
        };
        assert_eq!(native.record.as_obs().unwrap().get(&key), Some(&data));
    }
}

#[test]
fn bad_blocks_have_context_no_partial_samples_and_recover_at_modern_marker() {
    for bad in [
        epoch("0.0000000", Some("INVALID"), None),
        epoch("0.0000000", None, None).replace("-12.500", " NOT_A_NUMBER"),
        epoch("0.0000000", None, None).replace("-12.500", "     NaN"),
        epoch("0.0000000", None, None).replacen("  0  1", "  0 XX", 1),
        epoch("0.0000000", None, None).replace("G03", "Gé3"),
    ] {
        let text = synthetic_obs("4.02", &(bad + &epoch("1.0000000", None, None)));
        let all = collect(&text);
        assert_eq!(all.items.len(), 2);
        let ObsOutcome::Rejected { context, .. } = &all.items[0].outcome else {
            panic!("{:?}", all.items)
        };
        assert_eq!(context.flag, Some(EpochFlag::Ok));
        let ObsOutcome::Observation { data, .. } = &all.items[1].outcome else {
            panic!("{:?}", all.items)
        };
        assert_eq!(data.signals.len(), 2);
        assert!(data.clock.is_none());
    }
}

#[test]
fn exact_precision_and_unrepresentable_precision_are_not_merged() {
    let body = epoch("0.1234567", None, Some("01000"))
        + &epoch("1.1234567", None, Some("00001"))
        + &epoch("2.0000000", None, None);
    let all = collect(&synthetic_obs("4.02", &body));
    assert_eq!(all.items.len(), 3);
    let ObsOutcome::Observation { key, .. } = &all.items[0].outcome else {
        panic!()
    };
    assert_eq!(
        key.epoch,
        Epoch::from_str("2024-05-10T00:00:00.123456701 GPST").unwrap()
    );
    let ObsOutcome::Rejected { error, context } = &all.items[1].outcome else {
        panic!()
    };
    assert!(matches!(error, ParsingError::EpochPrecision));
    assert_eq!(context.epoch, None);
    assert_eq!(context.flag, Some(EpochFlag::Ok));
}

#[test]
fn blank_time_events_and_header_revisions_control_following_decode() {
    let body = epoch("0.0000000", None, None)
        + &event(2, &[record("MOVING", "COMMENT")])
        + &event(3, &[record("NEW SITE", "MARKER NAME")])
        + &event(4, &[record("G    2 C1C L1C", "SYS / # / OBS TYPES")])
        + &format!(
            "> 2024 05 10 00 00  0.0000000  5  1\n{}\n",
            record("EXTERNAL", "COMMENT")
        )
        + &epoch("1.0000000", None, None);
    let all = collect(&synthetic_obs("3.04", &body));
    assert_eq!(all.items.len(), 6);
    for (index, revision) in [(0, 0), (1, 0), (2, 1), (3, 2), (4, 2), (5, 2)] {
        assert_eq!(all.items[index].source.block_index, Some(index));
        assert_eq!(all.items[index].header_revision, revision);
    }
    let ObsOutcome::Event(ObsEvent::Special {
        epoch,
        kind,
        records,
        ..
    }) = &all.items[1].outcome
    else {
        panic!()
    };
    assert_eq!(*epoch, None);
    assert_eq!(*kind, SpecialEvent::AntennaMoving);
    assert_eq!(records, &[record("MOVING", "COMMENT")]);
    let ObsOutcome::Event(ObsEvent::Special {
        header_update: Some(header),
        ..
    }) = &all.items[3].outcome
    else {
        panic!()
    };
    assert_eq!(
        header.obs.as_ref().unwrap().codes[&Constellation::GPS],
        [
            Observable::from_str("C1C").unwrap(),
            Observable::from_str("L1C").unwrap()
        ]
    );
    let ObsOutcome::Observation { data, .. } = &all.items[5].outcome else {
        panic!()
    };
    assert_eq!(
        data.signals[0].observable,
        Observable::from_str("C1C").unwrap()
    );
    assert_eq!(data.signals[0].value, -12.5);
    // The single-header Rinex container rejects an unrepresentable state change.
    assert!(matches!(
        super::parse(&synthetic_obs("3.04", &body)),
        Err(ParsingError::ObsHeaderUpdate(_))
    ));
}

#[test]
fn failed_header_update_is_atomic_terminal_and_retains_prefix() {
    for update in [
        record("G    2 C1C", "SYS / # / OBS TYPES"),
        record("UNKNOWN", "UNSUPPORTED HEADER"),
        record("INVALID", "APPROX POSITION XYZ"),
        record("INVALID", "MARKER TYPE"),
    ] {
        let text = synthetic_obs(
            "3.04",
            &(epoch("0.0000000", None, None)
                + &event(4, &[update])
                + &epoch("1.0000000", None, None)),
        );
        let mut reader = ObservationReader::new(Cursor::new(text.as_bytes())).unwrap();
        let before = reader.header().clone();
        assert!(reader.next().unwrap().is_ok());
        let error = reader.next().unwrap().unwrap_err();
        assert_eq!(
            error.context.flag,
            Some(EpochFlag::HeaderInformationFollows)
        );
        assert_eq!(reader.header(), &before);
        assert_eq!(reader.status(), ReadStatus::Failed);
        assert!(reader.next().is_none());
        let failure = ObservationReader::new(Cursor::new(text.as_bytes()))
            .unwrap()
            .collect_records(ObsDuplicatePolicy::Preserve)
            .unwrap_err();
        assert_eq!(failure.partial.items.len(), 1);
        assert_eq!(failure.partial.status, ReadStatus::Failed);
    }
}

#[test]
fn external_events_require_time_and_modern_descriptors_require_a_marker() {
    let text = synthetic_obs("3.04", &event(5, &[record("EXTERNAL", "COMMENT")]));
    let mut reader = ObservationReader::new(Cursor::new(text.as_bytes())).unwrap();
    let error = reader.next().unwrap().unwrap_err();
    assert!(matches!(error.cause, ParsingError::EpochFormat));
    assert_eq!(error.context.flag, Some(EpochFlag::ExternalEvent));
    assert_eq!(reader.status(), ReadStatus::Failed);
    let malformed = epoch("0.0000000", None, None).replacen('>', " ", 1);
    let all = collect(&synthetic_obs(
        "3.04",
        &(malformed + &epoch("1.0000000", None, None)),
    ));
    assert!(matches!(
        all.items[0].outcome,
        ObsOutcome::Rejected {
            error: ParsingError::EpochFormat,
            ..
        }
    ));
    assert!(matches!(
        all.items[1].outcome,
        ObsOutcome::Observation { .. }
    ));
}

#[test]
fn header_updates_preserve_antenna_coordinates_and_fixed_width_eccentricities() {
    // Synthetic 3F14.4 boundary: the negative second field fills all 14 columns.
    let fields = format!("{:14.4}{:14.4}{:14.4}", 1.25, -12345678.9012, 3.75);
    for reverse in [false, true] {
        let mut records = vec![
            record(&fields, "ANTENNA: DELTA H/E/N"),
            record(&fields, "ANTENNA: DELTA X/Y/Z"),
            record(&fields, "APPROX POSITION XYZ"),
            record(
                &format!("{:20}{:20}", "NEW SERIAL", "NEW MODEL"),
                "ANT # / TYPE",
            ),
        ];
        if reverse {
            records.reverse();
        }
        let body = event(4, &records) + &epoch("0.0000000", None, None);
        let all = collect(&synthetic_obs("3.04", &body));
        let ObsOutcome::Event(ObsEvent::Special {
            header_update: Some(header),
            ..
        }) = &all.items[0].outcome
        else {
            panic!("{:?}", all.items);
        };
        let antenna = header.rcvr_antenna.as_ref().unwrap();
        assert_eq!(antenna.height, Some(1.25));
        assert_eq!(antenna.eastern, Some(-12345678.9012));
        assert_eq!(antenna.northern, Some(3.75));
        assert_eq!(antenna.coords, Some((1.25, -12345678.9012, 3.75)));
        assert_eq!(header.rx_position, antenna.coords);
        assert_eq!(antenna.model, "NEW MODEL");
    }
}

#[test]
fn compensation_updates_replace_the_previous_system_entry() {
    let mut records = Vec::new();
    for label in ["SYS / DCBS APPLIED", "SYS / PCVS APPLIED"] {
        for (system, program) in [("G", "OLD"), ("R", "UNCHANGED"), ("G", "NEW")] {
            records.push(record(
                &format!("{system} {program:<18}{:<40}", "source"),
                label,
            ));
        }
    }
    let all = collect(&synthetic_obs("3.04", &event(4, &records)));
    let ObsOutcome::Event(ObsEvent::Special {
        header_update: Some(header),
        ..
    }) = &all.items[0].outcome
    else {
        panic!()
    };
    assert_eq!(header.dcb_compensations.len(), 2);
    assert_eq!(header.pcv_compensations.len(), 2);
    for (system, program) in [
        (Constellation::GPS, "NEW"),
        (Constellation::Glonass, "UNCHANGED"),
    ] {
        assert_eq!(
            header
                .dcb_compensations
                .iter()
                .find(|entry| entry.constellation == system)
                .unwrap()
                .program,
            program
        );
        assert_eq!(
            header
                .pcv_compensations
                .iter()
                .find(|entry| entry.constellation == system)
                .unwrap()
                .program,
            program
        );
    }
}

#[test]
fn new_site_records_preserve_marker_fields_in_any_header_order() {
    let name = "NEW SITE WITH A NAME LONGER THAN TWENTY CHARACTERS";
    let records = [
        record("10118M123", "MARKER NUMBER"),
        record("GEODETIC", "MARKER TYPE"),
        record(name, "MARKER NAME"),
    ];
    let all = collect(&synthetic_obs("3.04", &event(3, &records)));
    let ObsOutcome::Event(ObsEvent::Special {
        header_update: Some(header),
        ..
    }) = &all.items[0].outcome
    else {
        panic!()
    };
    let marker = header.geodetic_marker.as_ref().unwrap();
    assert_eq!(marker.name, name);
    assert_eq!(marker.number().as_deref(), Some("10118M123"));
    assert_eq!(
        marker.marker_type,
        Some(crate::marker::MarkerType::Geodetic)
    );
}

#[test]
fn cycle_slips_and_power_failures_keep_their_distinct_meaning() {
    let slips = format!(
        "> 2024 05 10 00 00  0.0000000  6  1\nG03{:>14}  {:>14}  \n",
        "-2.000", "0.000"
    );
    let power = epoch("1.0000000", None, None).replacen("  0  1", "  1  1", 1);
    let all = collect(&observation_rinex(
        "3.04",
        "G    2 L1C L2W",
        &(slips + &power),
    ));
    let ObsOutcome::Event(ObsEvent::CycleSlip { slips, .. }) = &all.items[0].outcome else {
        panic!("{:?}", all.items)
    };
    assert_eq!(slips.len(), 2);
    assert_eq!(slips[0].value, -2.0);
    let ObsOutcome::Observation { key, .. } = &all.items[1].outcome else {
        panic!()
    };
    assert_eq!(key.flag, EpochFlag::PowerFailure);
}

#[test]
fn duplicate_collection_policy_is_explicit_and_source_order_survives() {
    let block = epoch("0.0000000", None, None);
    let text = synthetic_obs(
        "3.04",
        &(block.clone() + &block + &block.replace("-12.500", "-13.500")),
    );
    let all = collect(&text);
    assert_eq!(all.items.len(), 3);
    for (i, item) in all.items.iter().enumerate() {
        assert_eq!(item.source.block_index, Some(i));
    }
    let failure = ObservationReader::new(Cursor::new(text.as_bytes()))
        .unwrap()
        .collect_records(ObsDuplicatePolicy::Reject)
        .unwrap_err();
    assert_eq!(failure.partial.items.len(), 2);
    assert_eq!(failure.partial.status, ReadStatus::Reading);
    assert_eq!(failure.error.source.unwrap().block_index, Some(1));
}

#[test]
fn eof_truncation_unknown_state_and_limits_are_observable() {
    let truncated = epoch("0.0000000", None, None)
        .lines()
        .next()
        .unwrap()
        .to_owned();
    let all = collect(&synthetic_obs("3.04", &truncated));
    assert!(matches!(
        all.items[0].outcome,
        ObsOutcome::Rejected {
            error: ParsingError::ObsSatelliteCount {
                expected: 1,
                found: 0
            },
            ..
        }
    ));
    let unknown = epoch("0.0000000", None, None).replacen("  0  1", "  9  1", 1);
    let text = synthetic_obs("3.04", &unknown);
    let mut reader = ObservationReader::new(Cursor::new(text.as_bytes())).unwrap();
    assert!(matches!(
        reader.next().unwrap().unwrap_err().cause,
        ParsingError::EpochFlag
    ));
    assert_eq!(reader.status(), ReadStatus::Failed);
    assert!(reader.next().is_none());
    assert!(ObservationReader::with_options(
        Cursor::new(text.as_bytes()),
        ObservationReaderOptions {
            max_line_bytes: 10,
            max_block_bytes: 100
        }
    )
    .is_err());
}

#[test]
fn real_v2_and_crinex_v1_clock_values_match_original_columns() {
    for path in [
        "data/OBS/V2/delf0010_clock.21o",
        "data/CRNX/V1/delf0010_clock.21d",
    ] {
        // Source pair: the CRINEX fixture contains 3&-123456 (nanoseconds)
        // and 3&126298057858 (millimetre/millicycle fixed-point value).
        let input = std::fs::File::open(path).unwrap();
        let mut reader = ObservationReader::new(BufReader::new(input)).unwrap();
        let first = reader.next().unwrap().unwrap();
        let ObsOutcome::Observation { key, data, .. } = first.outcome else {
            panic!("{first:?}")
        };
        assert_eq!(key.epoch.to_string(), "2021-01-01T00:00:00 GPST");
        assert_eq!(data.clock.unwrap().offset_s, -0.000123456);
        assert_eq!(data.signals.len(), 21);
        assert_eq!(data.signals[0].sv, SV::from_str("G07").unwrap());
        assert_eq!(data.signals[0].value, 126_298_057.858);
    }
}

fn v2_header(body: &str) -> String {
    format!("{:>9}           OBSERVATION DATA    G                   RINEX VERSION / TYPE\n{}\n{}\n{}\n{body}", "2.11", record(&format!("{:6}{:>6}{:>6}", 2, "L1", "C1"), "# / TYPES OF OBSERV"), record("  2024     5    10     0     0    0.0000000     GPS", "TIME OF FIRST OBS"), record("", "END OF HEADER"))
}
fn v2_epoch(second: u8) -> String {
    format!(
        "{:26}  0  1G03\n{:>14}  {:>14}  \n",
        format!(" 24  5 10  0  0 {second:2}.0000000"),
        "-12.500",
        "0.000"
    )
}

#[test]
fn v2_blank_time_event_is_a_block_and_can_change_observable_count() {
    let body = v2_epoch(0)
        + &format!(
            "{:26}  4  1\n{}\n",
            "",
            record(&format!("{:6}{:>6}", 1, "C1"), "# / TYPES OF OBSERV")
        )
        + &format!(
            "{:26}  0  1G03\n{:>14}  \n",
            " 24  5 10  0  0  1.0000000", "25.000"
        );
    let all = collect(&v2_header(&body));
    assert_eq!(all.items.len(), 3);
    assert_eq!(all.items[1].source.block_index, Some(1));
    assert!(matches!(
        all.items[1].outcome,
        ObsOutcome::Event(ObsEvent::Special {
            epoch: None,
            header_update: Some(_),
            ..
        })
    ));
    let ObsOutcome::Observation { data, .. } = &all.items[2].outcome else {
        panic!("{:?}", all.items)
    };
    assert_eq!(data.signals.len(), 1);
    assert_eq!(
        data.signals[0].observable,
        Observable::from_str("C1").unwrap()
    );
    assert_eq!(data.signals[0].value, 25.0);
}

#[test]
fn real_crinex_v3_has_owned_measurements_and_no_false_eof() {
    let input = std::fs::File::open("data/CRNX/V3/DUTH0630.22D").unwrap();
    let all = ObservationReader::new(BufReader::new(input))
        .unwrap()
        .collect_records(ObsDuplicatePolicy::Preserve)
        .unwrap();
    assert_eq!(all.status, ReadStatus::EndOfInput);
    assert_eq!(all.items.len(), 3);
    let ObsOutcome::Observation { data, .. } = &all.items[0].outcome else {
        panic!()
    };
    assert_eq!(data.signals[0].value, 20_243_517.560);
    assert_eq!(data.signals[0].sv, SV::from_str("G01").unwrap());
    assert_eq!(all.items[0].source.coordinate, SourceCoordinate::CrinexText);
}

#[test]
fn truncated_crinex_and_gzip_corruption_are_terminal() {
    let text = std::fs::read_to_string("data/CRNX/V3/DUTH0630.22D").unwrap();
    let end = text.find("\n3&20243517560").unwrap();
    let partial = &text[..end];
    let mut reader = ObservationReader::new(Cursor::new(partial.as_bytes())).unwrap();
    assert!(reader.next().unwrap().is_err());
    assert_eq!(reader.status(), ReadStatus::Failed);
    assert!(reader.next().is_none());
    #[cfg(feature = "flate2")]
    {
        use std::io::Write;
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(source_observations().as_bytes()).unwrap();
        let mut bytes = encoder.finish().unwrap();
        let valid = ObservationReader::from_gzip(Cursor::new(&bytes))
            .unwrap()
            .collect_records(ObsDuplicatePolicy::Preserve)
            .unwrap();
        assert_eq!(
            valid.items[0].source.coordinate,
            SourceCoordinate::GzipDecodedText
        );
        let i = bytes.len() - 8;
        bytes[i] ^= 1;
        match ObservationReader::from_gzip(Cursor::new(&bytes)) {
            Err(_) => {},
            Ok(reader) => assert!(reader
                .collect_records(ObsDuplicatePolicy::Preserve)
                .is_err()),
        }
    }
}

struct Counted {
    input: Cursor<Vec<u8>>,
    consumed: Rc<Cell<usize>>,
}
impl Read for Counted {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.input.read(buf)?;
        self.consumed.set(self.consumed.get() + n);
        Ok(n)
    }
}
impl BufRead for Counted {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.input.fill_buf()
    }
    fn consume(&mut self, n: usize) {
        self.input.consume(n);
        self.consumed.set(self.consumed.get() + n);
    }
}

#[test]
fn pull_reader_does_not_decode_the_tail_and_is_fused_after_failure() {
    let bad = epoch("1.0000000", None, None).replacen("  0  1", "  9  1", 1);
    let text = synthetic_obs(
        "3.04",
        &(epoch("0.0000000", None, None) + &bad + &epoch("2.0000000", None, None)),
    );
    let consumed = Rc::new(Cell::new(0));
    let input = Counted {
        input: Cursor::new(text.clone().into_bytes()),
        consumed: consumed.clone(),
    };
    let mut reader = ObservationReader::new(input).unwrap();
    assert!(reader.next().unwrap().is_ok());
    assert!(consumed.get() < text.len());
    assert!(reader.next().unwrap().is_err());
    let after = consumed.get();
    assert!(reader.next().is_none());
    assert_eq!(consumed.get(), after);
}

#[test]
fn comments_are_located_without_occupying_block_indices() {
    let comment = record("BETWEEN", "COMMENT");
    let body =
        epoch("0.0000000", None, None) + &format!("{comment}\n") + &epoch("1.0000000", None, None);
    let all = collect(&synthetic_obs("3.04", &body));
    assert_eq!(all.items.len(), 3);
    assert_eq!(all.items[0].source.block_index, Some(0));
    assert!(matches!(&all.items[1].outcome, ObsOutcome::Comment(line) if line == &comment));
    assert_eq!(all.items[1].source.block_index, None);
    assert_eq!(all.items[2].source.block_index, Some(1));
}

struct FailingInput {
    bytes: Cursor<Vec<u8>>,
    fail_at: usize,
}
impl Read for FailingInput {
    fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        let remaining = self.fail_at.saturating_sub(self.bytes.position() as usize);
        if remaining == 0 {
            return Err(std::io::Error::other("synthetic input failure"));
        }
        let n = remaining.min(output.len());
        self.bytes.read(&mut output[..n])
    }
}

#[test]
fn io_failure_preserves_yielded_prefix_context_and_terminal_state() {
    let a = epoch("0.0000000", None, None);
    let b = epoch("1.0000000", None, None);
    let text = synthetic_obs("3.04", &(a + &b));
    let fail_at =
        text.find("> 2024 05 10 00 00  1.0000000").unwrap() + b.lines().next().unwrap().len() + 1;
    let input = BufReader::new(FailingInput {
        bytes: Cursor::new(text.into_bytes()),
        fail_at,
    });
    let error = ObservationReader::new(input)
        .unwrap()
        .collect_records(ObsDuplicatePolicy::Preserve)
        .unwrap_err();
    assert_eq!(error.partial.items.len(), 1);
    assert_eq!(error.partial.status, ReadStatus::Failed);
    assert_eq!(error.error.context.flag, Some(EpochFlag::Ok));
    assert_eq!(
        error.error.context.epoch.unwrap().to_string(),
        "2024-05-10T00:00:01 GPST"
    );
    assert_eq!(error.error.source.unwrap().block_index, Some(1));
    assert!(matches!(error.error.cause, ParsingError::InputIo(_)));
}

#[test]
fn corrupt_compressed_numeric_fields_are_not_treated_as_missing() {
    let original = std::fs::read_to_string("data/CRNX/V3/DUTH0630.22D").unwrap();
    for corrupt in ["BAD_FIELD", "3&NOT_A_NUMBER", "17"] {
        let text = original.replacen("3&20243517560", corrupt, 1);
        let error = ObservationReader::new(Cursor::new(text.as_bytes()))
            .unwrap()
            .collect_records(ObsDuplicatePolicy::Preserve)
            .unwrap_err();
        assert_eq!(error.partial.status, ReadStatus::Failed);
        assert!(error.partial.items.is_empty());
        assert!(matches!(error.error.cause, ParsingError::CRINEX(_)));
    }
    let shortened = original.replacen(
        "G01G03G04G09G17G19G21G22G31G32R01R02R08R09R10R17R23R24",
        "G01",
        1,
    );
    assert!(ObservationReader::new(Cursor::new(shortened.as_bytes()))
        .unwrap()
        .collect_records(ObsDuplicatePolicy::Preserve)
        .is_err());
}

#[test]
fn compressed_schema_change_initialises_new_positional_kernels() {
    // Synthetic CRINEX3 with one source-format numeric kernel per epoch.
    let body = "> 2024 05 10 00 00 00.0000000  0  1      G03\n\n3&20000000123\n".to_owned()
        + &event(4, &[record("G    1 L1C", "SYS / # / OBS TYPES")])
        + "> 2024 05 10 00 00 30.0000000  0  1      G03\n\n3&109000001234\n";
    let text = format!(
        "{}\n{}\n{}",
        record(
            "3.0                 COMPACT RINEX FORMAT",
            "CRINEX VERS   / TYPE"
        ),
        record(
            "RNX2CRX ver.4.1.0                       31-Jan-25 09:24",
            "CRINEX PROG / DATE"
        ),
        observation_rinex("3.04", "G    1 C1C", &body)
    );
    let all = collect(&text);
    assert_eq!(all.items.len(), 3);
    let ObsOutcome::Observation { data, .. } = &all.items[2].outcome else {
        panic!("{:?}", all.items)
    };
    assert_eq!(
        data.signals[0].observable,
        Observable::from_str("L1C").unwrap()
    );
    assert_eq!(data.signals[0].value, 109_000_001.234);
    assert_eq!(all.items[2].header_revision, 1);
}

#[test]
fn blank_satellite_rows_remain_observable_as_missing_measurements() {
    let body = "> 2024 05 10 00 00 00.0000000  0  1\nG03\n";
    let all = collect(&synthetic_obs("3.04", body));
    let ObsOutcome::Observation {
        data, satellites, ..
    } = &all.items[0].outcome
    else {
        panic!()
    };
    assert!(data.signals.is_empty());
    assert_eq!(satellites, &[SV::from_str("G03").unwrap()]);
}

#[test]
fn comments_cannot_bypass_block_limits_and_error_lines_include_comments() {
    let mut body = epoch("0.0000000", None, None);
    for _ in 0..20 {
        body.push_str(&format!("{}\n", record("PADDING", "COMMENT")));
    }
    let text = synthetic_obs("3.04", &body);
    let mut reader = ObservationReader::with_options(
        Cursor::new(text.as_bytes()),
        ObservationReaderOptions {
            max_line_bytes: 100,
            max_block_bytes: 500,
        },
    )
    .unwrap();
    assert!(reader.next().unwrap().is_err());
    let bad = format!(
        "> 2024 05 10 00 00 00.0000000  0  1\n{}\nG\n",
        record("BEFORE BAD ROW", "COMMENT")
    );
    let all = collect(&synthetic_obs("3.04", &bad));
    assert!(matches!(
        all.items[0].outcome,
        ObsOutcome::Rejected {
            error: ParsingError::ObsSignalParsing {
                line: 3,
                byte_column: 1
            },
            ..
        }
    ));
}

#[test]
fn empty_compressed_epoch_and_error_in_following_epoch_keep_good_prefix() {
    let prefix = format!(
        "{}\n{}\n{}",
        record(
            "3.0                 COMPACT RINEX FORMAT",
            "CRINEX VERS   / TYPE"
        ),
        record(
            "RNX2CRX ver.4.1.0                       31-Jan-25 09:24",
            "CRINEX PROG / DATE"
        ),
        observation_rinex("3.04", "G    1 C1C", "")
    );
    let text = prefix + "> 2024 05 10 00 00 00.0000000  0  0\n\n> 2024 05 10 00 00 30.0000000  0  1      G03\n\nBAD_VALUE\n";
    let error = ObservationReader::new(Cursor::new(text.as_bytes()))
        .unwrap()
        .collect_records(ObsDuplicatePolicy::Preserve)
        .unwrap_err();
    assert_eq!(error.partial.items.len(), 1);
    assert!(
        matches!(&error.partial.items[0].outcome, ObsOutcome::Observation { data, satellites, .. } if data.signals.is_empty() && satellites.is_empty())
    );
    assert_eq!(error.partial.status, ReadStatus::Failed);
}

#[test]
fn invalid_calendar_and_two_digit_year_boundary_are_not_guessed() {
    let bad = epoch("0.0000000", None, None).replacen("2024", "0000", 1);
    let all = collect(&synthetic_obs(
        "3.04",
        &(bad + &epoch("1.0000000", None, None)),
    ));
    assert!(matches!(
        all.items[0].outcome,
        ObsOutcome::Rejected {
            error: ParsingError::EpochFormat,
            context: ObsContext { epoch: None, .. }
        }
    ));
    let body = v2_epoch(0).replacen(" 24 ", " 79 ", 1);
    let all = collect(&v2_header(&body));
    let ObsOutcome::Observation { key, .. } = &all.items[0].outcome else {
        panic!()
    };
    assert_eq!(key.epoch.to_string(), "2079-05-10T00:00:00 GPST");
}

#[test]
fn mislabeled_header_records_stop_instead_of_corrupting_following_schema() {
    let body = format!(
        "> 2024 05 10 00 00 00.0000000  0  1\n{}\n",
        record("G    2 C1C L1C", "SYS / # / OBS TYPES")
    ) + &epoch("1.0000000", None, None);
    let text = synthetic_obs("3.04", &body);
    let mut reader = ObservationReader::new(Cursor::new(text.as_bytes())).unwrap();
    let before = reader.header().clone();
    assert!(matches!(
        reader.next().unwrap().unwrap_err().cause,
        ParsingError::ObsStream(_)
    ));
    assert_eq!(reader.header(), &before);
    assert_eq!(reader.status(), ReadStatus::Failed);
}
