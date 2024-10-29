use binex::prelude::{
    EphemerisFrame, Epoch, Message, MonumentGeoMetadata, MonumentGeoRecord, Record, Solutions,
    TemporalSolution,
};

#[test]
fn big_endian_message() {
    for msg in [
        Message::new(
            true,
            false,
            false,
            Record::new_monument_geo(
                MonumentGeoRecord::new(
                    Epoch::from_gpst_seconds(61.75),
                    MonumentGeoMetadata::RNX2BIN,
                )
                .with_climatic_info("climatic")
                .with_comment("comment #1")
                .with_comment("#comment 2")
                .with_geophysical_info("geophysics")
                .with_user_id("Custom ID#"),
            ),
        ),
        Message::new(
            true,
            false,
            false,
            Record::new_solutions(
                Solutions::new(Epoch::from_gpst_seconds(60.100)).with_pvt_ecef_wgs84(
                    1.0,
                    2.0,
                    3.0,
                    4.0,
                    5.0,
                    6.0,
                    TemporalSolution {
                        offset_s: 1.0,
                        drift_s_s: None,
                    },
                ),
            ),
        ),
        Message::new(
            true,
            false,
            false,
            Record::new_solutions(
                Solutions::new(Epoch::from_gpst_seconds(60.100)).with_pvt_ecef_wgs84(
                    1.0,
                    2.0,
                    3.0,
                    4.0,
                    5.0,
                    6.0,
                    TemporalSolution {
                        offset_s: 1.0,
                        drift_s_s: Some(2.0),
                    },
                ),
            ),
        ),
    ] {
        let mut buf = [0; 1024];
        msg.encode(&mut buf).unwrap();

        let parsed = Message::decode(&buf).unwrap();
        assert_eq!(parsed, msg);
    }
}
