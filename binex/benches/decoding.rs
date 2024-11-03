use binex::prelude::{
    EphemerisFrame, Epoch, Message, Meta, MonumentGeoMetadata, MonumentGeoRecord, Record,
    TimeResolution,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[allow(unused_must_use)]
pub fn criterion_benchmark(c: &mut Criterion) {
    let mut meta = Meta::default();
    meta.big_endian = true;

    let t0 = Epoch::from_gpst_seconds(10.0);
    let geo_meta = MonumentGeoMetadata::RNX2BIN;

    let mut record = MonumentGeoRecord::default()
        .with_comment("This is a test")
        .with_climatic_info("basic info")
        .with_geophysical_info("another field")
        .with_user_id("Test");

    record.epoch = t0;
    record.meta = geo_meta;

    let record = Record::new_monument_geo(record);
    let msg = Message::new(meta, TimeResolution::QuarterSecond, record);

    let mut buf = [0; 256];
    msg.encode(&mut buf, 256).unwrap();

    c.bench_function("decoding-00", |b| {
        b.iter(|| {
            black_box(Message::decode(&buf).unwrap());
        })
    });

    let record = Record::new_ephemeris_frame(EphemerisFrame::GPSRaw(Default::default()));
    let msg = Message::new(meta, TimeResolution::QuarterSecond, record);

    let mut buf = [0; 256];
    msg.encode(&mut buf, 256).unwrap();

    c.bench_function("decoding-01-00", |b| {
        b.iter(|| {
            black_box(Message::decode(&buf).unwrap());
        })
    });

    let record = Record::new_ephemeris_frame(EphemerisFrame::GPS(Default::default()));
    let msg = Message::new(meta, TimeResolution::QuarterSecond, record);

    let mut buf = [0; 256];
    msg.encode(&mut buf, 256).unwrap();

    c.bench_function("decoding-01-01", |b| {
        b.iter(|| {
            black_box(Message::decode(&buf).unwrap());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
