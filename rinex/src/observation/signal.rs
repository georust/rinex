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
    pub fn to_ionosphere_model(&self, rhs: &Self) -> Option<TEC> {
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
