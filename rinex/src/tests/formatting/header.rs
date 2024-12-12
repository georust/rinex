use crate::{
    prelude::{Header, Version},
    tests::formatting::Utf8Buffer,
};

#[test]
fn obs_gps_header_formatting() {
    let mut buf = Utf8Buffer::new();
    let mut header = Header::basic_obs()
        .with_version(Version::new(3, 5))
        .with_comment("test comment")
        .with_comment(
            "super long comment that needs to overflow the 60c limitation for testing purposes",
        )
        .with_general_information("TEST-PGM", "RUNBY", "AGENCY")
        .with_receiver(
            Receiver::default()
                .with_model("TEST RX")
                .with_sn("TEST SN-RX")
                .with_firmware("RX-FW"),
        )
        .with_receiver_antenna(
            Antenna::default()
                .with_model("TEST ANT")
                .with_sn("TEST SN-ANT")
                .with_firmware("ANT-FW"),
        )
        .with_constellation(Constellation::GPS);

    header.format(&mut buf).unwrap();

    let content = buf.to_ascii_utf8();
    let mut nb_tests = 0;

    for (nth, line) in content.lines() {
        nb_tests += 1;
    }

    assert!(nb_tests, 10);
}

#[test]
fn crinex_mixed_header_formatting() {
    let mut buf = Utf8Buffer::new(1024);
    let mut header = Header::basic_crinex();
    header.format(&mut buf).unwrap();

    let content = buf.to_ascii_utf8();
}
