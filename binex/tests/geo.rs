use binex::prelude::{Epoch, Message, MonumentGeoMetadata, MonumentGeoRecord, Record};

#[test]
fn geo_message() {
    let big_endian = false;
    let reversed = false;
    let enhanced_crc = false;
    let t = Epoch::from_gpst_seconds(10.0 + 0.75);

    let geo = MonumentGeoRecord::new(t, MonumentGeoMetadata::IGS).with_comment("Hello");

    let record = Record::new_monument_geo(geo);
    let msg = Message::new(big_endian, enhanced_crc, reversed, record);

    let mut buf = [0; 24];
    msg.encode(&mut buf).unwrap();

    assert_eq!(
        buf,
        [0xC2, 0, 13, 0, 0, 0, 0, 43, 2, 0, 5, 72, 101, 108, 108, 111, 0, 0, 0, 0, 0, 0, 0, 0]
    );

    let geo = MonumentGeoRecord::new(t, MonumentGeoMetadata::IGS)
        .with_comment("Hello")
        .with_comment("World");

    let record = Record::new_monument_geo(geo);
    let msg = Message::new(big_endian, enhanced_crc, reversed, record);

    let mut buf = [0; 32];
    msg.encode(&mut buf).unwrap();

    assert_eq!(
        buf,
        [
            0xC2, 0, 20, 0, 0, 0, 0, 43, 2, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114, 108,
            100, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]
    );

    let geo = MonumentGeoRecord::new(t, MonumentGeoMetadata::IGS)
        .with_comment("Hello")
        .with_comment("World");

    let record = Record::new_monument_geo(geo);
    let msg = Message::new(big_endian, enhanced_crc, reversed, record);

    let mut buf = [0; 32];
    msg.encode(&mut buf).unwrap();

    assert_eq!(
        buf,
        [
            0xC2, 0, 20, 0, 0, 0, 0, 43, 2, 0, 5, 72, 101, 108, 108, 111, 0, 5, 87, 111, 114, 108,
            100, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]
    );

    let geo = MonumentGeoRecord::new(t, MonumentGeoMetadata::IGS)
        .with_comment("Hello")
        .with_climatic_info("Clim");

    let record = Record::new_monument_geo(geo);
    let msg = Message::new(big_endian, enhanced_crc, reversed, record);

    let mut buf = [0; 32];
    msg.encode(&mut buf).unwrap();

    assert_eq!(
        buf,
        [
            0xC2, 0, 19, 0, 0, 0, 0, 43, 2, 0, 5, 72, 101, 108, 108, 111, 14, 4, 67, 108, 105, 109,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]
    );
}
