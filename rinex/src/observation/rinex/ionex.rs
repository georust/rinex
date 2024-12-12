use crate::{
    ionex::TEC,
    observation::SignalObservation,
    prelude::{Epoch, Observable, Rinex, SV},
};

use std::collections::HashMap;

/// The [TEC] estimate is indexed by [TECKey] when
/// calculated from Observation RINEX.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TECKey {
    /// [SV] is the signal source
    pub sv: SV,
    /// [Epoch] of estimation
    pub epoch: Epoch,
    /// RHS signal
    pub rhs: Observable,
    /// Reference (pivot) signal
    pub reference: Observable,
}

impl Rinex {
    /// Calculates Total Electron Contect (TEC) for each SV signal observed by this
    /// Observation RINEX. The TEC is evaluated using a dual frequency model:
    /// this limits the method to dual frequency Observation RINEX.
    /// Returns [TEC] sorted per [TECKey]
    pub fn observation_dual_phase_ionosphere_tec(&self) -> HashMap<TECKey, TEC> {
        let mut t = Epoch::default();
        let mut ret = HashMap::new();
        let mut phases = Vec::<SignalObservation>::new();

        for (k, ph) in self.phase_range_sampling_ok_iter() {
            if k.epoch > t && !phases.is_empty() {
                for ph_i in phases.iter() {
                    for ph_j in phases.iter() {
                        if let Some(tec) = ph_i.tec_estimate(&ph_j) {
                            ret.insert(
                                TECKey {
                                    sv: ph_i.sv,
                                    epoch: t,
                                    rhs: ph_j.observable.clone(),
                                    reference: ph_i.observable.clone(),
                                },
                                tec,
                            );
                        }
                    }
                }
                phases.clear();
            }

            t = k.epoch;
            phases.push(ph.clone());
        }

        ret
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::{Carrier, Epoch, Rinex};
    use hifitime::Unit;

    #[test]
    fn dual_phase_range_tec_estimation() {
        let gamma = 1.0 / 40.308;

        let path = format!(
            "{}/../test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx",
            env!("CARGO_MANIFEST_DIR"),
        );

        let rinex = Rinex::from_file(&path).unwrap();

        let t0 = Epoch::from_gregorian_utc_at_midnight(2021, 12, 21);
        let t1 = t0 + 30.0 * Unit::Second;
        let t2 = t1 + 30.0 * Unit::Second;

        let f_l1 = Carrier::L1.frequency().powi(2);
        let f_l2 = Carrier::L2.frequency().powi(2);
        let f_l5 = Carrier::L5.frequency().powi(2);

        let tec = rinex.observation_dual_phase_ionosphere_tec();

        panic!("{:#?}", tec);

        let mut tests_passed = 0;

        for (k, tec) in tec {
            match (k.epoch, k.sv, k.reference, k.rhs) {
                (t0, g01, l2s, l1c) => {
                    assert_eq!(tec.tec(), gamma * f_l1 * f_l2 / (f_l1 - f_l2) * (1.0 - 2.0));
                    tests_passed += 1;
                },
                (t0, g01, l1c, l2w) => {},
                (t0, g01, l1c, l5q) => {},
                (t0, g07, l1c, l2s) => {},
                (t0, g07, l1c, l2w) => {},
                (t0, g07, l1c, l5q) => {},
                (t0, e11, l1c, l5q) => {},
                (t0, e11, l1c, l6q) => {},
                (t0, e11, l1c, l7q) => {},
                (t1, g01, l1c, l2s) => {},
                (t1, g01, l1c, l2w) => {},
                (t1, g01, l1c, l5q) => {},
                (t1, g07, l1c, l2s) => {},
                (t1, g07, l1c, l2w) => {},
                (t1, g07, l1c, l5q) => {},
                (t1, e11, l1c, l5q) => {},
                (t1, e11, l1c, l6q) => {},
                (t1, e11, l1c, l7q) => {},
                (t2, g01, l1c, l2s) => {},
                (t2, g01, l1c, l2w) => {},
                (t2, g01, l1c, l5q) => {},
                (t2, g07, l1c, l2s) => {},
                (t2, g07, l1c, l2w) => {},
                (t2, g07, l1c, l5q) => {},
                (t2, e11, l1c, l5q) => {},
                (t2, e11, l1c, l6q) => {},
                (t2, e11, l1c, l7q) => {},
                _ => {},
            }
        }

        assert_eq!(tests_passed, 1);
    }
}
