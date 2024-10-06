//! Observation RINEX content
use crate::{
    observable::Observable,
    observation::{LliFlags, SNR},
    prelude::SV,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Observation RINEX content
#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Observation {
    /// Local clock state sometimes described,
    /// mostly dependent on receiver context and capabilities.
    ClockOffset(f64),
    /// [SignalObservation] (most common entry) describes signal sampling results.
    Signal(SignalObservation),
    /// [ObservationEvent] describes (usually abnormal) events
    Event(ObservationEvent),
}

impl Default for ObservationEntry {
    fn default() -> Self {
        Self::Signal(Default::default())
    }
}

impl Observation {
    /// Builds new [ObservationEvent]
    pub fn new_event(ev: ObservationEvent) -> Self {
        Self::Event(ev)
    }
    /// Builds new [SignalObservation] from value. Unit is [Observable] dependent
    pub fn new_signal(
        sv: SV,
        observable: Observable,
        value: f64,
        snr: Option<SNR>,
        lli: Option<LliFlags>,
    ) -> Self {
        Self::Signal(SignalObservation::new(sv, observable, value, snr, lli))
    }
    /// Builds new ClockOffset [Observation] from value in [s]
    pub fn new_clock(value_s: f64) -> Self {
        Self::ClockOffset(value_s)
    }
}

/// Signal Observation
#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SignalObservation {
    /// [SV] in sight (= signal source)
    pub sv: SV,
    /// [Observable] (= physics)
    pub observable: Observable,
    /// Measurement, unit depends on [Observable]
    pub value: f64,
    /// S/N Ratio
    pub snr: Option<SNR>,
    /// Lock loss indicator as [LliFlags], mostly relevant for [Observable::Phase] tracking
    pub lli: Option<LliFlags>,
}

impl SignalObservation {
    /// Builds new [SignalObservation]
    pub fn new(
        sv: SV,
        observable: Observable,
        value: f64,
        snr: Option<SNR>,
        lli: Option<LliFlags>,
    ) -> SignalObservation {
        SignalObservation {
            sv,
            observable,
            value,
            lli,
            snr,
        }
    }
    /// [SignalObservation] is defined as "OK" if
    ///   * [SNR] is declared as .strong() (checkout def)
    ///   * [LliFlags::OK_OR_UNKNOWN] is asserted
    /// NB: when both are missing, this will still return "OK", which makes
    /// this method compatible with "poor" quality RINEX where flags are regurlarly missing.
    /// Prefer [Self::is_ok_strict] otherwise.
    pub fn is_ok(self) -> bool {
        let lli_ok = self.lli.unwrap_or(LliFlags::OK_OR_UNKNOWN) == LliFlags::OK_OR_UNKNOWN;
        let snr_ok = self.snr.unwrap_or_default().strong();
        lli_ok && snr_ok
    }
    /// [SignalObservation] is strictly defined as "OK" if both
    /// [SNR] and [LliFlags] were passed and marked as .strong() and [OK_OR_UNKNOWN].
    /// If any of those is missing, this will not return OK.
    /// This is particularly useful when processing signal Phase.
    /// [Self::is_ok] otherwise.
    pub fn is_ok_strict(self) -> bool {
        if let Some(lli) = self.lli {
            if let Some(snr) = self.snr {
                lli == LliFlags::OK_OR_UNKNOWN && snr.strong()
            } else {
                false
            }
        } else {
            false
        }
    }
    /// Returns true if [SignalObservation] if both [LliFlags] and [SNR]
    /// are present and [SNR] is above or equal to minimal S/N ratio.
    pub fn is_ok_min_snr(&self, min_snr: SNR) -> bool {
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
    /// Helper to convert Pseudo Range to meters. Use this
    /// along Pseudo Range Observations.
    /// See p17;p18 of the RINEX specs.
    /// ## Inputs
    ///   - clk_offset: ClockOffset [m of delay]
    ///   - sv_clk_offset: SV ClockOffset [m of delay]
    /// ## Output
    ///   - range [m]. NB This does not account for other possible biases (simple macro)
    pub fn pseudo_range_to_range_m(&self, clk_offset: f64, sv_clk_offset: f64) -> f64 {
        self.value + 299_792_458.0_f64 * (clk_offset - sv_clk_offset)
    }
}

/// [ObservationEvent] describe hardware and context releated events
/// (usually abnormal) which aim at answering the need of accurate phase tracking
/// and precise navigation.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ObservationEvent {
    /// Back to normal. If previous errors were noted, they have been fixed
    AllGood,
    /// Antenna displacement is not compatible with accurate phase tracking
    AntennaBeingMoved,
}
