use crate::ionex::{Quantized as QuantizedIonex, TEC};
use crate::prelude::{
    Carrier, Epoch, EpochFlag, LliFlags, ObsKey, Observable, Rinex, SignalObservation, SV,
};
use itertools::Itertools;

/// The [TEC] estimate is indexed by [TECKey] when
/// calculated from Observation RINEX.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TECKey {
    /// [Epoch] of estimation
    pub epoch: Epoch,
    /// [SV] is the signal source
    pub sv: SV,
}

/// Supported signal [Combination]s
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Combination {
    /// Geometry Free (GF) combination (same physics)
    GeometryFree,
    /// Ionosphere Free (IF) combination (same physics)
    IonosphereFree,
    /// Wide Lane (Wl) combination (same physics)
    WideLane,
    /// Narrow Lane (Nl) combination (same physics)
    NarrowLane,
    /// Melbourne-Wübbena (MW) combination (cross-mixed physics)
    MelbourneWubbena,
}

/// Definition of a [SignalCombination]
#[derive(Debug, Clone)]
pub struct SignalCombination {
    /// [Combination] that was formed
    pub combination: Combination,
    /// Reference [Observable]
    pub reference: Observable,
    /// Left hand side (compared) [Observable]
    pub lhs: Observable,
    /// Value, unit is meters of delay of the (lhs - reference) frequency
    pub value: f64,
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
                    let same_sv = phase_a.sv == phase_b.sv;
                    let synchronous = k_a.epoch == k_b.epoch;
                    let phase_a_is_l1 = phase_a.observable.to_string().contains('1');
                    let phase_b_is_lj = !phase_a.observable.to_string().contains('1');
                    if synchronous && same_sv && phase_a_is_l1 && phase_b_is_lj {
                        if let Ok(carrier_a) = phase_a.observable.carrier(phase_a.sv.constellation)
                        {
                            if let Ok(carrier_b) =
                                phase_b.observable.carrier(phase_b.sv.constellation)
                            {
                                let f_a = carrier_a.frequency().powi(2);
                                let f_b = carrier_b.frequency().powi(2);
                                let tec_u = f_a * f_b / (f_a - f_b) / 40.308
                                    * (phase_a.value - phase_b.value);
                                let exponent = QuantizedIonex::find_exponent(tec_u);
                                let tec = TEC::new(tec_u);
                                let key = TECKey {
                                    epoch: k_a.epoch,
                                    sv: phase_a.sv,
                                };
                                Some((key, tec))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }),
        )
    }
}
