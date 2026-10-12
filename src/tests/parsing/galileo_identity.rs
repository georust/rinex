//! IGS NAV3 regressions. Excerpts are selected in memory from the original file.
use crate::{
    navigation::{NavFrameType, NavMessageType},
    prelude::*,
    record::ParsingDiagnosticKind,
};
use std::io::{BufReader, BufWriter};

// Original source lines, SHA-256:
// 5e51e8bfeee1218e5bf2fc0192b5647f805f549039650dbfb44595b9d4451855.
fn real() -> String {
    super::navigation_excerpt(
        "BRDC00IGS_R_20241310000_01D_MN.rnx",
        &[(1, 93), (318, 333), (8634, 8665)],
    )
}

fn read(text: &str) -> (Rinex, Vec<crate::record::ParsingDiagnostic>) {
    Rinex::parse_with_diagnostics(&mut BufReader::new(text.as_bytes())).unwrap()
}

#[test]
fn real_nav3_galileo_keeps_four_sources_and_gps_keys() {
    let (rinex, diagnostics) = read(&real());
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let records = rinex.record.as_nav().unwrap();
    assert_eq!(records.len(), 6);
    let mut galileo: Vec<_> = records
        .iter()
        .filter(|(k, _)| k.sv.constellation == Constellation::Galileo)
        .collect();
    galileo.sort_by_key(|(k, _)| k.galileo_data_sources);
    for ((key, frame), (source, msg, clock, transmit)) in galileo.iter().zip([
        (258, NavMessageType::FNAV, 1.261251745746E-04, 431500.0),
        (513, NavMessageType::INAV, 1.261241850443E-04, 431464.0),
        (516, NavMessageType::INAV, 1.261241850440E-04, 431464.0),
        (517, NavMessageType::INAV, 1.261241850443E-04, 431455.0),
    ]) {
        assert_eq!(key.galileo_data_sources, Some(source));
        assert_eq!(key.msgtype, msg);
        assert_eq!(key.frmtype, NavFrameType::Ephemeris);
        assert_eq!(key.epoch.to_string(), "2024-05-09T23:40:00 GST");
        let eph = frame.as_ephemeris().unwrap();
        assert_eq!(eph.clock_bias, clock);
        assert_eq!(eph.get_orbit_f64("source"), Some(source as f64));
        assert_eq!(eph.get_orbit_f64("t_tm"), Some(transmit));
    }
    assert_eq!(galileo.len(), 4);
    for (key, _) in records
        .iter()
        .filter(|(k, _)| k.sv.constellation == Constellation::GPS)
    {
        assert_eq!(key.galileo_data_sources, None);
        assert_eq!(key.msgtype, NavMessageType::LNAV);
    }
    #[cfg(feature = "serde")]
    {
        let gps_key = records
            .keys()
            .find(|k| k.sv.constellation == Constellation::GPS)
            .unwrap();
        let mut old = serde_json::to_value(gps_key).unwrap();
        assert!(old.get("galileo_data_sources").is_none());
        let back: crate::navigation::NavKey = serde_json::from_value(old.take()).unwrap();
        assert_eq!(&back, gps_key);
    }
}

#[test]
fn real_galileo_nav3_and_nav4_write_read_preserve_every_frame() {
    let (original, _) = read(&real());
    for version in [
        Version::new(3, 4),
        Version::new(4, 0),
        Version::new(4, 1),
        Version::new(4, 2),
    ] {
        let mut rinex = original.clone();
        rinex.header.version = version;
        let mut writer = BufWriter::new(Vec::new());
        rinex.format(&mut writer).unwrap();
        let bytes = writer.into_inner().unwrap();
        let output = std::str::from_utf8(&bytes).unwrap();
        let lines: Vec<_> = output.lines().collect();
        let mut galileo_count = 0;
        for (i, line) in lines.iter().enumerate() {
            let start = if version.major < 4 && line.starts_with('E') {
                i
            } else if version.major >= 4 && line.starts_with("> EPH E") {
                i + 1
            } else {
                continue;
            };
            galileo_count += 1;
            // Independent layout expectations: V3 Table A8 versus V4 Table A13.
            let lengths = if version.major < 4 {
                [80, 80, 80]
            } else {
                [61, 80, 23]
            };
            for (offset, length) in [5, 6, 7].into_iter().zip(lengths) {
                assert_eq!(lines[start + offset].len(), length, "version {version}");
            }
        }
        assert_eq!(galileo_count, 4);
        let (mut back, diagnostics) = read(output);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(back.record, original.record, "version {version}");

        // The reverse conversion must restore the actual NAV3 spare columns.
        back.header.version = Version::new(3, 4);
        let mut writer = BufWriter::new(Vec::new());
        back.format(&mut writer).unwrap();
        let bytes = writer.into_inner().unwrap();
        let (back, diagnostics) = read(std::str::from_utf8(&bytes).unwrap());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(
            back.record, original.record,
            "version {version} back to NAV3"
        );
    }
}

#[test]
fn real_galileo_nav3_spares_do_not_shift_fields() {
    let source = super::navigation_excerpt(
        "BRDC00IGS_R_20241310000_01D_MN.rnx",
        &[(1, 93), (8634, 8641)],
    );
    let (original, diagnostics) = read(&source);
    assert!(diagnostics.is_empty());
    // Synthetic contents of the four actual spares defined by V3 Table A8.
    let mut lines: Vec<_> = source.lines().map(str::to_owned).collect();
    let spare = format!("{:>19}", "SPARE");
    lines[98].truncate(61);
    lines[98].push_str(&spare);
    lines[100].truncate(23);
    lines[100].push_str(&spare.repeat(3));
    let (rinex, diagnostics) = read(&lines.join("\n"));
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(rinex.record, original.record);
    let (key, frame) = rinex.record.as_nav().unwrap().first_key_value().unwrap();
    assert_eq!(key.galileo_data_sources, Some(258));
    assert_eq!(key.transmission_time.unwrap().seconds(), 431500.0);
    assert_eq!(
        frame.as_ephemeris().unwrap().get_orbit_f64("sisa"),
        Some(3.12)
    );
}

#[test]
fn same_full_galileo_key_still_reports_real_content_conflicts() {
    // Synthetic repetition/change of an unchanged real source block.
    let real = real();
    let lines: Vec<_> = real.split_inclusive('\n').collect();
    let header = lines[..93].concat();
    let block = lines[109..117].concat();
    let changed = block.replacen("1.261251745746E-04", "1.361251745746E-04", 1);
    let text = format!("{header}{block}{block}{changed}");
    let (rinex, diagnostics) = read(&text);
    assert_eq!(rinex.record.as_nav().unwrap().len(), 1);
    assert_eq!(diagnostics.len(), 2);
    assert!(matches!(
        diagnostics[0].kind,
        ParsingDiagnosticKind::DuplicateNavigation {
            conflicting: false,
            previous_record_line: 1,
            ..
        }
    ));
    assert!(matches!(
        diagnostics[1].kind,
        ParsingDiagnosticKind::DuplicateNavigation {
            conflicting: true,
            previous_record_line: 9,
            ..
        }
    ));
    assert_eq!(diagnostics[0].record_line, 9);
    assert_eq!(diagnostics[1].record_line, 17);
    let (_, first) = rinex.record.as_nav().unwrap().first_key_value().unwrap();
    assert_eq!(first.as_ephemeris().unwrap().clock_bias, 1.361251745746E-04);
    assert_eq!(
        Rinex::parse(&mut BufReader::new(text.as_bytes()))
            .unwrap()
            .record,
        rinex.record
    );
}

#[test]
fn missing_or_mixed_galileo_sources_do_not_guess_a_message() {
    let real = real();
    let lines: Vec<_> = real.split_inclusive('\n').collect();
    let header = lines[..93].concat();
    let block = lines[109..117].concat();
    for source in [
        "                   ",
        " 0.000000000000E+00",
        " 2.590000000000E+02",
    ] {
        let changed = block.replace(" 2.580000000000E+02", source);
        let (rinex, diagnostics) = read(&format!("{header}{changed}"));
        assert!(rinex.record.as_nav().unwrap().is_empty());
        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].kind,
            ParsingDiagnosticKind::NavigationFailure(ParsingError::NavGalileoDataSources)
        ));
    }
}
#[test]
fn galileo_clock_reference_bits_are_mutually_exclusive() {
    let excerpt = super::navigation_excerpt(
        "BRDC00IGS_R_20241310000_01D_MN.rnx",
        &[(1, 93), (8634, 8641)],
    );
    // Synthetic boundary: 770 = F/NAV bit 1 plus BOTH clock bits 8 and 9.
    let changed = excerpt.replace(" 2.580000000000E+02", " 7.700000000000E+02");
    assert_ne!(changed, excerpt);
    let (rinex, diagnostics) =
        Rinex::parse_with_diagnostics(&mut BufReader::new(changed.as_bytes())).unwrap();
    assert!(rinex.record.as_nav().unwrap().is_empty());
    assert!(matches!(
        diagnostics.as_slice(),
        [crate::record::ParsingDiagnostic {
            kind: ParsingDiagnosticKind::NavigationFailure(ParsingError::NavGalileoDataSources),
            ..
        }]
    ));
}

#[test]
fn invalid_galileo_sources_do_not_saturate_and_reserved_bits_survive() {
    let source = super::navigation_excerpt(
        "BRDC00IGS_R_20241310000_01D_MN.rnx",
        &[(1, 93), (8634, 8641)],
    );
    for value in [258.5, -258.0, 4294967296.0, 770.0, 259.0, 263.0, 0.0] {
        let field = format!("{value:19.12E}");
        let text = source.replace(" 2.580000000000E+02", &field);
        let (r, d) = read(&text);
        assert!(r.record.as_nav().unwrap().is_empty(), "{value}");
        assert_eq!(d.len(), 1);
        assert!(matches!(
            d[0].kind,
            ParsingDiagnosticKind::NavigationFailure(_)
        ));
    }
    let text = source.replace(" 2.580000000000E+02", " 2.820000000000E+02"); // 258 + reserved bits 3/4
    let (r, d) = read(&text);
    assert!(d.is_empty());
    let (k, f) = r.record.as_nav().unwrap().first_key_value().unwrap();
    assert_eq!(k.galileo_data_sources, Some(282));
    assert_eq!(
        f.as_ephemeris().unwrap().get_orbit_f64("source"),
        Some(282.0)
    );
    for version in [Version::new(3, 4), Version::new(3, 5), Version::new(4, 2)] {
        let mut output = r.clone();
        output.header.version = version;
        let mut writer = BufWriter::new(Vec::new());
        output.format(&mut writer).unwrap();
        let bytes = writer.into_inner().unwrap();
        let (back, diagnostics) = read(std::str::from_utf8(&bytes).unwrap());
        assert!(diagnostics.is_empty());
        assert_eq!(back.record, r.record);
    }
    // A direct NAV4 mismatch uses the same unchanged F/NAV payload.
    let block = source.lines().skip(93).collect::<Vec<_>>().join("\n");
    let text = super::navigation_rinex("4.00", &format!("> EPH E02 INAV\n{block}\n"));
    let (_, d) = read(&text);
    assert!(matches!(
        d[0].kind,
        ParsingDiagnosticKind::NavigationFailure(ParsingError::NavGalileoDataSources)
    ));
}

#[test]
fn galileo_rounding_is_shared_by_parser_manual_frames_and_writer() {
    use crate::navigation::{gal::GalDataSource, NavKey, OrbitItem};
    let source = super::navigation_excerpt(
        "BRDC00IGS_R_20241310000_01D_MN.rnx",
        &[(1, 93), (8634, 8641)],
    );
    let (original, _) = read(&source);
    let (key, frame) = original.record.as_nav().unwrap().first_key_value().unwrap();
    // Synthetic derivatives of the real source field; expectations follow
    // nearest-integer conversion and Table A8's signal bits.
    for (value, bits, message) in [
        (258.4, 258, NavMessageType::FNAV),
        (257.5, 258, NavMessageType::FNAV),
        (516.4, 516, NavMessageType::INAV),
        (515.5, 516, NavMessageType::INAV),
        (516.5, 517, NavMessageType::INAV),
    ] {
        let text = source.replace(" 2.580000000000E+02", &format!("{value:19.12E}"));
        let (parsed, diagnostics) = read(&text);
        assert!(diagnostics.is_empty(), "{value}: {diagnostics:?}");
        let (parsed_key, parsed_frame) = parsed.record.as_nav().unwrap().first_key_value().unwrap();
        assert_eq!(parsed_key.galileo_data_sources, Some(bits));
        assert_eq!(parsed_key.msgtype, message);
        assert_eq!(
            parsed_frame.as_ephemeris().unwrap().get_orbit_f64("source"),
            Some(bits as f64)
        );
        for item in [
            OrbitItem::F64(value),
            OrbitItem::U32(bits),
            OrbitItem::GalDataSource(GalDataSource::from_bits(bits).unwrap()),
        ] {
            let mut changed = frame.clone();
            changed
                .as_mut_ephemeris()
                .unwrap()
                .orbits
                .insert("source".into(), item);
            let rebuilt = NavKey::from_frame(
                Version::new(3, 4),
                key.epoch,
                key.sv,
                NavMessageType::LNAV,
                key.subtype,
                &changed,
            )
            .unwrap();
            assert_eq!(rebuilt, *parsed_key);
            let mut output = original.clone();
            *output.record.as_mut_nav().unwrap() = [(rebuilt, changed)].into_iter().collect();
            output.header.version = Version::new(4, 2);
            let mut writer = BufWriter::new(Vec::new());
            output.format(&mut writer).unwrap();
            let bytes = writer.into_inner().unwrap();
            let (back, diagnostics) = read(std::str::from_utf8(&bytes).unwrap());
            assert!(diagnostics.is_empty());
            assert_eq!(back.record, parsed.record);
        }
    }
    for value in [
        290.0,
        770.0,
        259.0,
        0.0,
        -0.1,
        4294967296.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        let mut changed = frame.clone();
        changed
            .as_mut_ephemeris()
            .unwrap()
            .orbits
            .insert("source".into(), OrbitItem::F64(value));
        assert!(
            matches!(
                NavKey::from_frame(
                    Version::new(3, 4),
                    key.epoch,
                    key.sv,
                    key.msgtype,
                    key.subtype,
                    &changed,
                ),
                Err(ParsingError::NavGalileoDataSources)
            ),
            "{value}"
        );
        let mut writer = BufWriter::new(Vec::new());
        assert!(matches!(
            changed.as_ephemeris().unwrap().format(
                &mut writer,
                key.sv,
                Version::new(4, 2),
                key.msgtype,
                false,
            ),
            Err(crate::FormattingError::NavUnrepresentableIdentity)
        ));
        assert!(writer.into_inner().unwrap().is_empty());
    }
    let mut changed = frame.clone();
    changed.as_mut_ephemeris().unwrap().orbits.insert(
        "source".into(),
        OrbitItem::GalDataSource(GalDataSource::from_bits_retain(290)),
    );
    assert!(matches!(
        NavKey::from_frame(
            Version::new(4, 2),
            key.epoch,
            key.sv,
            key.msgtype,
            key.subtype,
            &changed,
        ),
        Err(ParsingError::NavGalileoDataSources)
    ));
    let mixed = source
        .lines()
        .skip(93)
        .collect::<Vec<_>>()
        .join("\n")
        .replace(" 2.580000000000E+02", " 2.590000000000E+02");
    let (unsupported, diagnostics) = read(&super::navigation_rinex("3.02", &mixed));
    assert!(unsupported.record.as_nav().unwrap().is_empty());
    assert_eq!(diagnostics.len(), 1);
}

#[test]
#[cfg(feature = "serde")]
fn galileo_e5b_inav_uses_the_correct_public_and_serialized_name() {
    use crate::navigation::gal::GalDataSource;
    assert_eq!(
        serde_json::to_value(GalDataSource::INAV_E5B_I).unwrap(),
        "INAV_E5B_I"
    );
    assert_eq!(
        GalDataSource::from_name("INAV_E5B_I"),
        Some(GalDataSource::INAV_E5B_I)
    );
    assert_eq!(GalDataSource::from_name("FNAV_E5B_I"), None);
}

#[test]
#[cfg(feature = "qc")]
fn navigation_merge_inserts_distinct_keys_and_keeps_left_on_conflict() {
    let (original, _) = read(&real());
    let mut lhs = original.clone();
    lhs.record
        .as_mut_nav()
        .unwrap()
        .retain(|k, _| k.galileo_data_sources == Some(258));
    let mut rhs = original;
    for f in rhs.record.as_mut_nav().unwrap().values_mut() {
        f.as_mut_ephemeris().unwrap().clock_bias += 0.001;
    }
    use crate::prelude::qc::Merge;
    lhs.merge_mut(&rhs).unwrap();
    let record = lhs.record.as_nav().unwrap();
    assert_eq!(record.len(), 6);
    let (k, f) = record
        .iter()
        .find(|(k, _)| k.galileo_data_sources == Some(258))
        .unwrap();
    assert_eq!(f.as_ephemeris().unwrap().clock_bias, 1.261251745746E-04);
    let _ = k;
}
