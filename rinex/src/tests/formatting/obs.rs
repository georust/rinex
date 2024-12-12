use std::str::FromStr;

use crate::{
    observation::Header,
    prelude::{Constellation, Observable},
    tests::formatting::{generic_formatted_lines_test, Utf8Buffer},
};

#[test]
fn mixed_header_formatting() {
    let gps = Constellation::GPS;
    let gal = Constellation::Galileo;

    let l1c = Observable::PhaseRange("L1C").to_string();
    let c1c = Observable::PhaseRange("C1C").to_string();
    let d1c = Observable::PhaseRange("D1C").to_string();
    let s1c = Observable::PhaseRange("S1C").to_string();

    let l2c = Observable::PhaseRange("L2C").to_string();
    let l2c = Observable::PhaseRange("L2C").to_string();
    let d2c = Observable::PhaseRange("D2C").to_string();
    let s2c = Observable::PhaseRange("S2C").to_string();

    let l5c = Observable::PhaseRange("L5C").to_string();
    let c5c = Observable::PhaseRange("C5C").to_string();
    let d5c = Observable::PhaseRange("D5C").to_string();
    let s5c = Observable::PhaseRange("S5C").to_string();

    let l1p = Observable::PhaseRange("L1P").to_string();
    let l2p = Observable::PhaseRange("L2P").to_string();
    let l1x = Observable::PhaseRange("L1X").to_string();
    let l2x = Observable::PhaseRange("L2X").to_string();

    let codes = HashMap::<Constellation, Vec<Observable>>::from(&[
        (
            gps,
            vec![
                l1c.clone(),
                c1c.clone(),
                d1c.clone(),
                s1c.clone(),
                l2c.clone(),
                c2c.clone(),
                d2c.clone(),
                s2c.clone(),
                l5c.clone(),
                c5c.clone(),
                d5c.clone(),
                s5c.clone(),
                l1p.clone(),
                l2p.clone(),
            ],
        ),
        (
            gal,
            vec![
                l1c.clone(),
                c1c.clone(),
                d1c.clone(),
                s1c.clone(),
                l2c.clone(),
                c2c.clone(),
                d2c.clone(),
                s2c.clone(),
                l5q.clone(),
                c5q.clone(),
                d5q.clone(),
                s5q.clone(),
                l1x.clone(),
                l2x.clone(),
            ],
        ),
    ]);

    let mut hd = Header::default()
        .with_timeof_first_obs(Epoch::from_str("2020-01-01T00:00:00 GPST").unwrap())
        .with_timeof_last_obs(Epoch::from_str("2020-01-01T00:00:00 GPST").unwrap())
        .with_codes(codes);

    hd.format(&mut buf).unwrap();
    let content = buf.to_ascii_utf8();

    let test_values = HashMap::from(&[(0, "test")]);

    generic_formatted_lines_test(content, test_values);
}

#[test]
fn high_precision_obs() {
    let gps = Constellation::GPS;
    let gal = Constellation::Galileo;
    let l1c = Observable::PhaseRange("L1C").to_string();
    let l5c = Observable::PhaseRange("L5C").to_string();
    let l1x = Observable::PhaseRange("L1X").to_string();
    let l5q = Observable::PhaseRange("L5Q").to_string();

    let mut hd = Header::default()
        .with_scaling((gps, l1c), 10)
        .with_scaling((gps, l5c), 20)
        .with_scaling((gal, l1x), 30)
        .with_scaling((gal, l5q), 40);

    hd.format(&mut buf).unwrap();
}
