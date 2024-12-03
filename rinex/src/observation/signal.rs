use crate::{
    observation::LliFlags,
    prelude::{ClockObservation, Observable, SNR, SV},
};

#[cfg(feature = "ionex")]
use crate::ionex::TEC;

/// [SignalObservation] is the result of sampling one signal at
/// one point in time, by a GNSS receiver.
#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SignalObservation {
    /// [SV] is the signal source
    pub sv: SV,
    /// Actual measurement. Unit depends on [Observable].
    pub value: f64,
    /// [Observable]
    pub observable: Observable,
    /// Lock loss indicator (when present)
    pub lli: Option<LliFlags>,
    /// SNR estimate (when present)
    pub snr: Option<SNR>,
}

impl SignalObservation {
    /// Builds new signal observation
    pub fn new(sv: SV, observable: Observable, value: f64) -> Self {
        Self {
            sv,
            observable,
            value,
            lli: None,
            snr: None,
        }
    }

    /// Copy and define [SNR]
    pub fn with_snr(&self, snr: SNR) -> Self {
        let mut s = self.clone();
        s.snr = Some(snr);
        s
    }

    /// [Observation] is said OK when
    ///  - If LLI is present it must match [LliFlags::OK_OR_UNKNOWN]
    ///  - If SNR is present, it must be [SNR::strong]
    ///  - NB: when both are missing, we still return OK.
    /// This allows method that Iterate over OK Epoch Data to consider
    /// data when SNR or LLI are missing.
    pub fn is_ok(self) -> bool {
        let lli_ok = self.lli.unwrap_or(LliFlags::OK_OR_UNKNOWN) == LliFlags::OK_OR_UNKNOWN;
        let snr_ok = self.snr.unwrap_or_default().strong();
        lli_ok && snr_ok
    }

    /// [Observation::is_ok] with additional SNR criteria to match (>=).
    /// SNR must then be present otherwise this is not OK.
    pub fn is_ok_snr(&self, min_snr: SNR) -> bool {
        if self
            .lli
            .unwrap_or(LliFlags::OK_OR_UNKNOWN)
            .intersects(LliFlags::OK_OR_UNKNOWN)
        {
            if let Some(snr) = self.snr {
                snr >= min_snr
            } else {
                false
            }
        } else {
            false
        }
    }

    #[cfg(feature = "ionex")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ionex")))]
    /// Calculates the Ionosphere [TEC] from two Phase or Code range observations.
    /// Currently limited to dual frequency Phase Range observations.
    pub fn tec_estimate(&self, rhs: &Self) -> Option<TEC> {
        let same_physics = self.observable.same_physics(&rhs.observable);
        let different_signals = self.observable != rhs.observable;
        let same_sv = self.sv == rhs.sv;
        let is_phase = self.observable.is_phase_range_observable();

        let carrier_1 = self.observable.carrier(self.sv.constellation);
        let carrier_2 = rhs.observable.carrier(self.sv.constellation);
        let both_ok = carrier_1.is_ok() && carrier_2.is_ok();

        if same_physics && is_phase && same_sv && different_signals && both_ok {
            let carrier_1 = carrier_1.unwrap();
            let carrier_2 = carrier_2.unwrap();
            let f_1 = carrier_1.frequency().powi(2);
            let f_2 = carrier_2.frequency().powi(2);
            if carrier_1.is_l1_pivot() && (f_1 != f_2) {
                let tec = 1.0 / 40.308 * f_1 * f_2 / (f_1 - f_2) * (self.value - rhs.value);
                Some(TEC::new(tec))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::{Carrier, Observable, SignalObservation, SV};
    use std::str::FromStr;
    #[test]
    fn test_dual_signal_tec_estimate() {
        let gamma = 1.0 / 40.308;

        let f_l1 = Carrier::L1.frequency().powi(2);
        let f_l2 = Carrier::L2.frequency().powi(2);
        let f_l5 = Carrier::L5.frequency().powi(2);

        let g01 = SV::from_str("G01").unwrap();
        let g02 = SV::from_str("G02").unwrap();

        let l1c = Observable::from_str("L1C").unwrap();
        let l2c = Observable::from_str("L2C").unwrap();
        let l5c = Observable::from_str("L5C").unwrap();

        let g01_l1c = SignalObservation::new(g01, l1c.clone(), 1.0);
        let g01_l2c = SignalObservation::new(g01, l2c.clone(), 2.0);
        let g01_l5c = SignalObservation::new(g01, l5c.clone(), 3.0);

        let g02_l1c = SignalObservation::new(g02, l1c.clone(), 4.0);
        let g02_l2c = SignalObservation::new(g02, l2c.clone(), 5.0);
        let g02_l5c = SignalObservation::new(g02, l5c.clone(), 6.0);

        // different SV: not ok!
        assert!(g01_l1c.tec_estimate(&g02_l2c).is_none());

        let tec = g01_l1c.tec_estimate(&g01_l2c).unwrap();

        assert_eq!(
            tec.tec(),
            gamma * f_l1 * f_l2 / (f_l1 - f_l2) * (g01_l1c.value - g01_l2c.value)
        );
    }
}
