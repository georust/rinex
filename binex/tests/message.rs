use binex::prelude::{
    EphemerisFrame, Epoch, GPSEphemeris, GPSRaw, Message, MonumentGeoMetadata, MonumentGeoRecord,
    Record, TimeResolution,
};

#[test]
fn test_crc8_geo() {
    let msg = Message::new(
        true,
        TimeResolution::QuarterSecond,
        false,
        false,
        Record::new_monument_geo(MonumentGeoRecord::new_igs(
            Epoch::from_gpst_seconds(61.25),
            "Great receiver",
            "Fancy antenna",
            "MARKERNAME",
            "MARKERNUMBER",
            "SITE",
            "SITENAME",
        )),
    );

    let mut buf = [0; 128];
    msg.encode(&mut buf).unwrap();

    let parsed = Message::decode(&buf).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn test_crc16_geo() {
    let msg = Message::new(
        true,
        TimeResolution::QuarterSecond,
        false,
        false,
        Record::new_monument_geo(
            MonumentGeoRecord::new_igs(
                Epoch::from_gpst_seconds(61.25),
                "Great receiver",
                "Fancy antenna",
                "MARKERNAME",
                "MARKERNUMBER",
                "SITE",
                "SITENAME",
            )
            .with_climatic_info("test")
            .with_comment("super")
            .with_geophysical_info("great")
            .with_project_name("project"),
        ),
    );

    let mut buf = [0; 128];
    msg.encode(&mut buf).unwrap();

    let parsed = Message::decode(&buf).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn test_crc8_eph() {
    let msg = Message::new(
        true,
        TimeResolution::QuarterSecond,
        false,
        false,
        Record::new_ephemeris_frame(EphemerisFrame::new_gps_raw(GPSRaw::default())),
    );

    let mut buf = [0; 128];
    msg.encode(&mut buf).unwrap();

    assert_eq!(buf[0], 226); // SYNC
    assert_eq!(buf[1], 1); // MID
    assert_eq!(buf[2], 78); // RLEN
                            // assert_eq!(buf[3 + 78], 79); // CRC TODO

    let parsed = Message::decode(&buf).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn test_crc16_eph() {
    let msg = Message::new(
        true,
        TimeResolution::QuarterSecond,
        false,
        false,
        Record::new_ephemeris_frame(EphemerisFrame::new_gps(GPSEphemeris::default())),
    );

    let mut buf = [0; 128];
    assert!(msg.encode(&mut buf).is_err());

    let mut buf = [0; 256];
    msg.encode(&mut buf).unwrap();

    let parsed = Message::decode(&buf).unwrap();
    assert_eq!(msg, parsed);
}
