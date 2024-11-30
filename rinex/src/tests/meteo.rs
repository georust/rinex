use crate::{
    prelude::Rinex,
    tests::toolkit::{generic_meteo_rinex_test, TimeFrame},
};

#[test]
fn v2_abvi0010_15m() {
    let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V2/abvi0010.15m";
    let rinex = Rinex::from_file(path).unwrap();

    generic_meteo_rinex_test(
        &rinex,
        None,
        "2.11",
        "PR, TD, HR, WS, WD, RI, HI",
        TimeFrame::from_erratic_csv(
            "
                        2015-01-01T00:00:00 UTC,
                        2015-01-01T00:01:00 UTC,
                        2015-01-01T00:02:00 UTC,
                        2015-01-01T00:03:00 UTC,
                        2015-01-01T00:04:00 UTC,
                        2015-01-01T00:05:00 UTC,
                        2015-01-01T00:06:00 UTC,
                        2015-01-01T00:07:00 UTC,
                        2015-01-01T00:08:00 UTC,
                        2015-01-01T00:09:00 UTC,
                        2015-01-01T09:00:00 UTC,
                        2015-01-01T09:01:00 UTC,
                        2015-01-01T09:02:00 UTC,
                        2015-01-01T09:03:00 UTC,
                        2015-01-01T09:04:00 UTC,
                        2015-01-01T19:25:00 UTC,
                        2015-01-01T19:26:00 UTC,
                        2015-01-01T19:27:00 UTC,
                        2015-01-01T19:28:00 UTC,
                        2015-01-01T19:29:00 UTC,
                        2015-01-01T19:30:00 UTC,
                        2015-01-01T19:31:00 UTC,
                        2015-01-01T19:32:00 UTC,
                        2015-01-01T19:33:00 UTC,
                        2015-01-01T19:34:00 UTC,
                        2015-01-01T19:35:00 UTC,
                        2015-01-01T19:36:00 UTC,
                        2015-01-01T19:37:00 UTC,
                        2015-01-01T19:38:00 UTC,
                        2015-01-01T19:39:00 UTC,
                        2015-01-01T19:40:00 UTC,
                        2015-01-01T19:41:00 UTC,
                        2015-01-01T19:42:00 UTC,
                        2015-01-01T19:43:00 UTC,
                        2015-01-01T19:44:00 UTC,
                        2015-01-01T19:45:00 UTC,
                        2015-01-01T19:46:00 UTC,
                        2015-01-01T19:47:00 UTC,
                        2015-01-01T19:48:00 UTC,
                        2015-01-01T19:49:00 UTC,
                        2015-01-01T19:50:00 UTC,
                        2015-01-01T19:51:00 UTC,
                        2015-01-01T19:52:00 UTC,
                        2015-01-01T19:53:00 UTC,
                        2015-01-01T19:54:00 UTC,
                        2015-01-01T22:55:00 UTC,
                        2015-01-01T22:56:00 UTC,
                        2015-01-01T22:57:00 UTC,
                        2015-01-01T22:58:00 UTC,
                        2015-01-01T22:59:00 UTC,
                        2015-01-01T23:00:00 UTC,
                        2015-01-01T23:01:00 UTC,
                        2015-01-01T23:01:00 UTC,
                        2015-01-01T23:02:00 UTC,
                        2015-01-01T23:09:00 UTC,
                        2015-01-01T23:10:00 UTC,
                        2015-01-01T23:11:00 UTC,
                        2015-01-01T23:12:00 UTC,
                        2015-01-01T23:13:00 UTC,
                        2015-01-01T23:14:00 UTC,
                        2015-01-01T23:15:00 UTC,
                        2015-01-01T23:16:00 UTC,
                        2015-01-01T23:17:00 UTC,
                        2015-01-01T23:18:00 UTC,
                        2015-01-01T23:19:00 UTC,
                        2015-01-01T23:20:00 UTC,
                        2015-01-01T23:21:00 UTC,
                        2015-01-01T23:52:00 UTC,
                        2015-01-01T23:53:00 UTC,
                        2015-01-01T23:54:00 UTC,
                        2015-01-01T23:55:00 UTC,
                        2015-01-01T23:56:00 UTC,
                        2015-01-01T23:57:00 UTC,
                        2015-01-01T23:58:00 UTC,
                        2015-01-01T23:59:00 UTC",
        ),
    );
}

#[test]
#[cfg(feature = "flate2")]
fn v3_pots00deu() {
    let path = env!("CARGO_MANIFEST_DIR").to_owned()
        + "/../test_resources/MET/V3/POTS00DEU_R_20232540000_01D_05M_MM.rnx.gz";
    let rinex = Rinex::from_gzip_file(path).unwrap();

    generic_meteo_rinex_test(
        &rinex,
        None,
        "3.05",
        "HR, PR, TD",
        TimeFrame::from_inclusive_csv("2023-09-11T00:00:00 UTC, 22023-09-11T23:55:00 UTC, 300s"),
    );
}

#[test]
fn v4_example_1() {
    let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V4/example1.txt";
    let rinex = Rinex::from_file(path).unwrap();

    generic_meteo_rinex_test(
        &rinex,
        None,
        "4.00",
        "PR, TD, HR",
        TimeFrame::from_inclusive_csv("2021-01-07T00:00:00 UTC, 2021-01-07T00:02:00 UTC, 30s"),
    );
}
