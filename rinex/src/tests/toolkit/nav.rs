use crate::{
    prelude::{Rinex, RinexType},
    tests::toolkit::{generic_rinex_test, sv_csv as sv_csv_parser, TimeFrame},
};

// use crate::navigation::KbModel;
// use crate::navigation::NgModel;
// use crate::prelude::{Constellation, Rinex};

// pub fn check_klobuchar_models(rinex: &Rinex, tuple: &[(Constellation, KbModel)]) {
//     let header = &rinex.header;
//     for (constell, model) in tuple {
//         let parsed = header.ionod_corrections.get(&constell);
//         assert!(parsed.is_some(), "missing KB Model for {}", constell);
//         let parsed = parsed.unwrap();
//         let parsed = parsed.as_klobuchar().unwrap();
//         assert_eq!(parsed, model);
//     }
// }

// pub fn check_nequick_g_models(rinex: &Rinex, tuple: &[(Constellation, NgModel)]) {
//     let header = &rinex.header;
//     for (constell, model) in tuple {
//         let parsed = header.ionod_corrections.get(&constell);
//         assert!(parsed.is_some(), "missing NG Model for {}", constell);
//         let parsed = parsed.unwrap();
//         let parsed = parsed.as_nequick_g().unwrap();
//         assert_eq!(parsed, model);
//     }
// }

pub fn generic_test(
    dut: &Rinex,
    version: &str,
    header_constellation: &str,
    time_frame: Option<TimeFrame>,
    sv_csv: &str,
    nb_ephemeris: usize,
) {
    assert!(dut.is_navigation_rinex());

    let _ = dut.record.as_nav().unwrap();

    let mut sv = sv_csv_parser(sv_csv);
    sv.sort();

    generic_rinex_test(
        dut,
        version,
        Some(header_constellation),
        RinexType::NavigationData,
        time_frame,
    );

    assert_eq!(dut.sv_iter().collect::<Vec<_>>(), sv);

    assert_eq!(dut.nav_ephemeris_frames_iter().count(), nb_ephemeris);
}

pub fn generic_comparison(dut: &Rinex, model: &Rinex) {
    let dut = dut.record.as_nav().unwrap();
    let model = model.record.as_nav().unwrap();

    for (k, v) in model.iter() {
        if let Some(dut_v) = dut.get(&k) {
            assert_eq!(v, dut_v);
        } else {
            panic!("missing data at {:?}", k);
        }
    }

    for (k, _) in dut.iter() {
        if model.get(&k).is_none() {
            panic!("found invalid data at {:?}", k);
        }
    }
}
