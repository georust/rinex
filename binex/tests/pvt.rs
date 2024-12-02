use binex::prelude::{Epoch, Message, Meta, Record, Solutions, TemporalSolution};

#[test]
fn test_pvt() {
    let mut meta = Meta::default();
    meta.big_endian = true;

    let msg = Message::new(
        meta,
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
    );

    let mut buf = [0; 1024];
    msg.encode(&mut buf, 1024).unwrap();

    let parsed = Message::decode(&buf).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_pvt_drift() {
    let mut meta = Meta::default();
    meta.big_endian = true;

    let msg = Message::new(
        meta,
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
    );

    let mut buf = [0; 1024];
    msg.encode(&mut buf, 1024).unwrap();

    let parsed = Message::decode(&buf).unwrap();
    assert_eq!(parsed, msg);
}
