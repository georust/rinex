use binex::prelude::{
    EphemerisFrame, Epoch, Message, MonumentGeoMetadata, MonumentGeoRecord, Record, TimeResolution,
};

#[test]
fn big_endian_message() {
    let t = Epoch::from_gpst_seconds(10.0);
    let geo_meta = MonumentGeoMetadata::RNX2BIN;

    for msg in [Message::new(
        true,
        TimeResolution::QuarterSecond,
        false,
        false,
        Record::new_monument_geo(
            MonumentGeoRecord::new(t, geo_meta)
                .with_climatic_info("climatic")
                .with_comment("comment #1")
                .with_comment("#comment 2")
                .with_geophysical_info("geophysics")
                .with_user_id("Custom ID#"),
        ),
    )] {
        let mut buf = [0; 1024];
        msg.encode(&mut buf).unwrap();

        let parsed = Message::decode(&buf).unwrap();

        assert_eq!(msg, parsed);
    }
}
