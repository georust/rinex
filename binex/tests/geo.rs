use binex::prelude::{
    Epoch, Message, Meta, MonumentGeoMetadata, MonumentGeoRecord, Record, TimeResolution,
};

#[test]
fn geo_message() {
    let mut meta = Meta::default();

    meta.reversed = false;
    meta.big_endian = true;
    meta.enhanced_crc = false;

    let t = Epoch::from_gpst_seconds(10.0 + 0.75);

    let time_res = TimeResolution::QuarterSecond;

    let mut geo = MonumentGeoRecord::default().with_comment("simple");

    geo.epoch = t;
    geo.meta = MonumentGeoMetadata::RNX2BIN;
    let record = Record::new_monument_geo(geo);
    let msg = Message::new(meta, time_res, record);

    let mut encoded = [0; 128];
    msg.encode(&mut encoded, 128).unwrap();

    let decoded = Message::decode(&encoded).unwrap();
    assert_eq!(decoded, msg);

    let geo = MonumentGeoRecord::new_igs(
        t,
        "Receiver Model",
        "Antenna Model",
        "Geodetic MARKER",
        "MARKER NUMBER",
        "Site location",
        "Site name",
    );

    let record = Record::new_monument_geo(geo);
    let msg = Message::new(meta, time_res, record);

    let mut encoded = [0; 128];
    msg.encode(&mut encoded, 128).unwrap();

    assert_eq!(encoded[0], 226);
    assert_eq!(encoded[1], 0);

    let decoded = Message::decode(&encoded).unwrap();
    assert_eq!(decoded, msg);
}
