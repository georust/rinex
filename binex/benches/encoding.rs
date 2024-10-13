use binex::prelude::{
    EphemerisFrame, Epoch, Message, MonumentGeoMetadata, MonumentGeoRecord, Record, TimeResolution,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[allow(unused_must_use)]
pub fn criterion_benchmark(c: &mut Criterion) {
    let mut buf = [0; 256];
    let t0 = Epoch::from_gpst_seconds(10.0);
    let meta = MonumentGeoMetadata::RNX2BIN;

    let record = MonumentGeoRecord::new(t0, meta)
        .with_comment("This is a test")
        .with_climatic_info("basic info")
        .with_geophysical_info("another field")
        .with_user_id("Test");

    let record = Record::new_monument_geo(record);
    let msg = Message::new(true, TimeResolution::QuarterSecond, false, false, record);

    c.bench_function("bnx00", |b| {
        b.iter(|| {
            black_box(msg.encode(&mut buf).unwrap());
        })
    });

    let record = Record::new_ephemeris_frame(EphemerisFrame::GPSRaw(Default::default()));
    let msg = Message::new(true, TimeResolution::QuarterSecond, false, false, record);

    c.bench_function("bnx01-00", |b| {
        b.iter(|| {
            black_box(msg.encode(&mut buf).unwrap());
        })
    });

    let record = Record::new_ephemeris_frame(EphemerisFrame::GPS(Default::default()));
    let msg = Message::new(true, TimeResolution::QuarterSecond, false, false, record);

    c.bench_function("bnx01-01", |b| {
        b.iter(|| {
            black_box(msg.encode(&mut buf).unwrap());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
