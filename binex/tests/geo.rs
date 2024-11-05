use binex::prelude::{Epoch, Message, Meta, MonumentGeoMetadata, MonumentGeoRecord, Record};

#[test]
fn geo_message() {
    let mut meta = Meta::default();

    meta.reversed = false;
    meta.big_endian = true;
    meta.enhanced_crc = false;

    let t = Epoch::from_gpst_seconds(10.0 + 0.75);

    let mut geo = MonumentGeoRecord::default().with_comment("simple");

    geo.epoch = t;
    geo.meta = MonumentGeoMetadata::RNX2BIN;
    let record = Record::new_monument_geo(geo);

    let msg = Message::new(meta, record);

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
    let msg = Message::new(meta, record);

    let mut encoded = [0; 128];
    msg.encode(&mut encoded, 128).unwrap();

    let decoded = Message::decode(&encoded).unwrap();
    assert_eq!(decoded, msg);
}
