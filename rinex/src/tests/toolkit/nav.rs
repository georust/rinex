use crate::navigation::KbModel;
use crate::navigation::NgModel;
use crate::prelude::{Constellation, Rinex};

pub fn check_klobuchar_models(rinex: &Rinex, tuple: &[(Constellation, KbModel)]) {
    let header = &rinex.header;
    for (constell, model) in tuple {
        let parsed = header.ionod_corrections.get(&constell);
        assert!(parsed.is_some(), "missing KB Model for {}", constell);
        let parsed = parsed.unwrap();
        let parsed = parsed.as_klobuchar().unwrap();
        assert_eq!(parsed, model);
    }
}

pub fn check_nequick_g_models(rinex: &Rinex, tuple: &[(Constellation, NgModel)]) {
    let header = &rinex.header;
    for (constell, model) in tuple {
        let parsed = header.ionod_corrections.get(&constell);
        assert!(parsed.is_some(), "missing NG Model for {}", constell);
        let parsed = parsed.unwrap();
        let parsed = parsed.as_nequick_g().unwrap();
        assert_eq!(parsed, model);
    }
}
