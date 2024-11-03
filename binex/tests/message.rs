use binex::prelude::{
    EphemerisFrame, Epoch, GPSEphemeris, GPSRaw, Message, Meta, MonumentGeoRecord, Record,
    TimeResolution,
};

#[test]
fn test_crc8_geo() {
    let mut meta = Meta::default();
    meta.big_endian = true;
    meta.reversed = false;
    meta.enhanced_crc = false;

    let msg = Message::new(
        meta,
        TimeResolution::QuarterSecond,
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
    msg.encode(&mut buf, 128).unwrap();

    let parsed = Message::decode(&buf).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn test_crc16_geo() {
    let mut meta = Meta::default();
    meta.big_endian = true;
    meta.reversed = false;
    meta.enhanced_crc = false;

    let msg = Message::new(
        meta,
        TimeResolution::QuarterSecond,
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
    msg.encode(&mut buf, 128).unwrap();

    let parsed = Message::decode(&buf).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn test_crc8_gps() {
    let mut meta = Meta::default();
    meta.big_endian = true;
    meta.reversed = false;
    meta.enhanced_crc = false;

    let msg = Message::new(
        meta,
        TimeResolution::QuarterSecond,
        Record::new_ephemeris_frame(EphemerisFrame::new_gps_raw(GPSRaw::default())),
    );

    let mut buf = [0; 128];
    msg.encode(&mut buf, 128).unwrap();

    assert_eq!(buf[0], 226); // SYNC
    assert_eq!(buf[1], 1); // MID
    assert_eq!(buf[2], 79); // RLEN

    let parsed = Message::decode(&buf).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn test_crc16_gps() {
    let mut meta = Meta::default();
    meta.big_endian = true;
    meta.reversed = false;
    meta.enhanced_crc = false;
    let msg = Message::new(
        meta,
        TimeResolution::QuarterSecond,
        Record::new_ephemeris_frame(EphemerisFrame::new_gps(GPSEphemeris::default())),
    );

    let mut encoded = [0; 128];
    assert!(msg.encode(&mut encoded, 128).is_err());

    let mut encoded = [0; 256];
    msg.encode(&mut encoded, 256).unwrap();

    let parsed = Message::decode(&encoded).unwrap();
    assert_eq!(msg, parsed);
}
