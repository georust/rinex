use crate::{
    prelude::Rinex,
    tests::toolkit::{generic_ionex_comparison, generic_ionex_test, TecPoint, TimeFrame},
};

use std::fs::remove_file;
use std::io::Write;
use std::path::Path;

#[test]
#[cfg(feature = "flate2")]
fn v1_ckmg0020_22i() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test_resources")
        .join("IONEX")
        .join("V1")
        .join("CKMG0020.22I.gz");
    let fullpath = path.to_string_lossy();

    let rinex = Rinex::from_gzip_file(fullpath.as_ref());
    assert!(rinex.is_ok(), "failed to parse IONEX/V1/CKMG0020.22I.gz");

    let dut = rinex.unwrap();

    generic_ionex_test(
        &dut,
        "1.0",
        false,
        "2022-01-02T00:00:00 UTC",
        "2022-01-03T00:00:00 UTC",
        87.5,
        -87.5,
        -2.5,
        -180.0,
        180.0,
        5.0,
        None,
        None,
        None,
        -1,
        6371.0,
        0.0,
        None,
        TimeFrame::from_inclusive_csv("2022-01-02T00:00:00 UTC, 2022-01-02T23:00:00 UTC, 1 hour"),
        vec![
            TecPoint {
                t: "2022-01-02T00:00:00 UTC",
                lat_ddeg: 87.5,
                lat_exponent: -1,
                long_ddeg: -180.0,
                long_exponent: -1,
                alt_km: 350.0,
                alt_exponent: 0, // null spacing
                tecu: 9.2,
            },
            TecPoint {
                t: "2022-01-02T00:00:00 UTC",
                lat_ddeg: -2.5,
                lat_exponent: 1,
                long_ddeg: -160.0,
                long_exponent: 1,
                alt_km: 350.0,
                alt_exponent: 0, // null spacing
                tecu: 38.3,
            },
            TecPoint {
                t: "2022-01-02T00:00:00 UTC",
                lat_ddeg: -2.5,
                lat_exponent: 1,
                long_ddeg: -155.0,
                long_exponent: 1,
                alt_km: 350.0,
                alt_exponent: 0, // null spacing
                tecu: 38.5,
            },
            TecPoint {
                t: "2022-01-02T00:00:00 UTC",
                lat_ddeg: -2.5,
                lat_exponent: 1,
                long_ddeg: -150.0,
                long_exponent: 1,
                alt_km: 350.0,
                alt_exponent: 0, // null spacing
                tecu: 38.5,
            },
            TecPoint {
                t: "2022-01-02T00:00:00 UTC",
                lat_ddeg: -2.5,
                lat_exponent: 1,
                long_ddeg: -145.0,
                long_exponent: 1,
                alt_km: 350.0,
                alt_exponent: 0, // null spacing
                tecu: 38.4,
            },
            TecPoint {
                t: "2022-01-02T00:00:00 UTC",
                lat_ddeg: -40.0,
                lat_exponent: 1,
                long_ddeg: -175.0,
                long_exponent: 1,
                alt_km: 350.0,
                alt_exponent: 0, // null spacing
                tecu: 21.6,
            },
        ],
    );

    dut.to_file("v1_ckmg0020_22i.txt").unwrap();

    let parsed = Rinex::from_file("v1_ckmg0020_22i.txt").unwrap();
    generic_ionex_comparison(parsed, dut);

    let _ = remove_file("v1_ckmg0020_22i.txt");
}

#[test]
#[cfg(feature = "flate2")]
fn v1_ckmg0090_12i() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test_resources")
        .join("IONEX")
        .join("V1")
        .join("CKMG0090.21I.gz");

    let fullpath = path.to_string_lossy();

    let rinex = Rinex::from_gzip_file(fullpath.as_ref());
    assert!(rinex.is_ok(), "failed to parse IONEX/V1CKMG0090.21I.gz");

    let dut = rinex.unwrap();

    generic_ionex_test(
        &dut,
        "1.0",
        false,
        "2021-01-09T00:00:00 UTC",
        "2021-01-10T00:00:00 UTC",
        87.5,
        -87.5,
        -2.5,
        -180.0,
        180.0,
        5.0,
        Some(350.0),
        Some(350.0),
        Some(0.0),
        -1,
        6371.0,
        0.0,
        None,
        TimeFrame::from_inclusive_csv("2021-01-09T00:00:00 UTC, 2021-01-09T23:00:00 UTC, 1 hour"),
        vec![],
    );
}

#[test]
#[cfg(feature = "flate2")]
fn v1_jplg0010_17i() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test_resources")
        .join("IONEX")
        .join("V1")
        .join("jplg0010.17i.gz");

    let fullpath = path.to_string_lossy();

    let rinex = Rinex::from_gzip_file(fullpath.as_ref());
    assert!(rinex.is_ok(), "failed to parse IONEX/jplg0010.17i.gz");

    let dut = rinex.unwrap();

    generic_ionex_test(
        &dut,
        "1.0",
        false,
        "2017-01-01T00:00:00 UTC",
        "2017-01-02T00:00:00 UTC",
        87.5,
        -87.5,
        -2.5,
        -180.0,
        180.0,
        5.0,
        Some(450.0),
        Some(450.0),
        Some(0.0),
        -1,
        6371.0,
        10.0,
        None,
        TimeFrame::from_inclusive_csv("2017-01-01T00:00:00 UTC, 2017-01-01T23:00:00 UTC, 2 hour"),
        vec![],
    );
}
