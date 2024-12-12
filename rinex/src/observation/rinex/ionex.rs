use crate::{
    ionex::{Quantized as QuantizedIonex, TEC},
    observation::{EpochFlag, LliFlags, ObsKey, SignalObservation},
    prelude::{Carrier, Epoch, Observable, Rinex, SV},
};

/// The [TEC] estimate is indexed by [TECKey] when
/// calculated from Observation RINEX.
#[derive(Debug, Clone, PartialEq)]
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
    pub fn observation_dual_phase_ionosphere_tec(
        &self,
    ) -> Box<dyn Iterator<Item = (TECKey, TEC)> + '_> {
        Box::new(
            self.phase_range_sampling_ok_iter()
                .zip(self.phase_range_sampling_ok_iter())
                .filter_map(|((k_a, phase_a), (k_b, phase_b))| {
                    let tec = phase_a.tec_estimate(&phase_b)?;
                    Some((
                        TECKey {
                            epoch: k_a.epoch,
                            sv: phase_a.sv,
                            rhs: phase_b.observable.clone(),
                            reference: phase_a.observable.clone(),
                        },
                        tec,
                    ))
                }),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::{Epoch, Rinex};
    use hifitime::Unit;

    // helper, calculates the TEC value from two observation
    fn tec_calc() -> f64 {
        0.0
    }

    #[test]
    fn dual_phase_range_tec_estimation() {
        let gamma = 1.0 / 40.308;

        let path = format!(
            "{}/test_resources/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx",
            env!("CARGO_MANIFEST_DIR"),
        );

        let rinex = Rinex::from_file(&path).unwrap();

        let t0 = Epoch::from_gregorian_utc_at_midnight(2021, 12, 21);
        let t1 = t0 + 30.0 * Unit::Second;
        let t2 = t1 + 30.0 * Unit::Second;

        let f_l1 = Carrier::L1.frequency().powi(2);
        let f_l2 = Carrier::L2.frequency().powi(2);
        let f_l5 = Carrier::L5.frequency().powi(2);

        let mut t0_g01_l1c_l2s_passed = false;
        let mut t0_g01_l1c_l2w_passed = false;
        let mut t0_g01_l1c_l5q_passed = false;
        let mut t0_g07_l1c_l2s_passed = false;
        let mut t0_g07_l1c_l2w_passed = false;
        let mut t0_g07_l1c_l5q_passed = false;
        let mut t0_e11_l1c_l5q_passed = false;
        let mut t0_e11_l1c_l6q_passed = false;
        let mut t0_e11_l1c_l7q_passed = false;

        let mut t1_g01_l1c_l2s_passed = false;
        let mut t1_g01_l1c_l2w_passed = false;
        let mut t1_g01_l1c_l5q_passed = false;
        let mut t1_g07_l1c_l2s_passed = false;
        let mut t1_g07_l1c_l2w_passed = false;
        let mut t1_g07_l1c_l5q_passed = false;
        let mut t1_e11_l1c_l5q_passed = false;
        let mut t1_e11_l1c_l6q_passed = false;
        let mut t1_e11_l1c_l7q_passed = false;

        let mut t2_g01_l1c_l2s_passed = false;
        let mut t2_g01_l1c_l2w_passed = false;
        let mut t2_g01_l1c_l5q_passed = false;
        let mut t2_g07_l1c_l2s_passed = false;
        let mut t2_g07_l1c_l2w_passed = false;
        let mut t2_g07_l1c_l5q_passed = false;
        let mut t2_e11_l1c_l5q_passed = false;
        let mut t2_e11_l1c_l6q_passed = false;
        let mut t2_e11_l1c_l7q_passed = false;

        let tec = rinex.observation_dual_phase_ionosphere_tec();

        for (k, tec) in tec {
            match (k.epoch, k.sv, k.reference, k.rhs) {
                (t0, g01, l1c, l2s) => {
                    assert_eq!(tec, gamma * f_l1 * f_l2 / (f_l1 - f_l2) * (1.0 - 2.0));
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

        let t0_test_passed = t0_g01_l1c_l2s_passed
            && t0_g01_l1c_l2w_passed
            && t0_g01_l1c_l5q_passed
            && t0_g07_l1c_l2s_passed
            && t0_g07_l1c_l2w_passed
            && t0_g07_l1c_l5q_passed
            && t0_e11_l1c_l5q_passed
            && t0_e11_l1c_l6q_passed
            && t0_e11_l1c_l7q_passed;

        let t1_test_passed = t1_g01_l1c_l2s_passed
            && t1_g01_l1c_l2w_passed
            && t1_g01_l1c_l5q_passed
            && t1_g07_l1c_l2s_passed
            && t1_g07_l1c_l2w_passed
            && t1_g07_l1c_l5q_passed
            && t1_e11_l1c_l5q_passed
            && t1_e11_l1c_l6q_passed
            && t1_e11_l1c_l7q_passed;

        let t2_test_passed = t2_g01_l1c_l2s_passed
            && t2_g01_l1c_l2w_passed
            && t2_g01_l1c_l5q_passed
            && t2_g07_l1c_l2s_passed
            && t2_g07_l1c_l2w_passed
            && t2_g07_l1c_l5q_passed
            && t2_e11_l1c_l5q_passed
            && t2_e11_l1c_l6q_passed
            && t2_e11_l1c_l7q_passed;

        let test_passed = t0_test_passed && t1_test_passed && t2_test_passed;
    }
}
