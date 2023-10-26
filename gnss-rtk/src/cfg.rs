use crate::SolverType;
use gnss::prelude::SNR;
use crate::model::Modeling;
use hifitime::prelude::TimeScale;

use std::str::FromStr;
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::Deserialize;

fn default_timescale() -> TimeScale {
    TimeScale::GPST
}

fn default_interp() -> usize {
    11
}

fn default_max_sv() -> usize {
    10
}

fn default_smoothing() -> bool {
    false
}

#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct RTKConfig {
    /// Time scale
    #[cfg_attr(feature = "serde", serde(default = "default_timescale"))]
    pub timescale: TimeScale,
    /// (Position) interpolation filter order.
    /// A minimal order must be respected for correct results.
    /// -  7 when working with broadcast ephemeris
    /// - 11 when working with SP3
    #[cfg_attr(feature = "serde", serde(default = "default_interp"))]
    pub interp_order: usize,
    /// Whether the solver is working in fixed altitude mode or not
    #[cfg_attr(feature = "serde", serde(default))]
    pub fixed_altitude: Option<f64>,
    /// PR code smoothing filter before moving forward
    #[cfg_attr(feature = "serde", serde(default = "default_smoothing"))]
    pub code_smoothing: bool,
    /// System Internal Delay, as defined by BIPM in
    /// "GPS Receivers Accurate Time Comparison" : the (frequency dependent)
    /// time delay introduced by the combination of:
    ///  + the RF cable (up to several nanoseconds)
    ///  + the distance between the antenna baseline and its APC:
    ///    a couple picoseconds, and is frequency dependent
    ///  + the GNSS receiver inner delay (hardware and frequency dependent)
    pub internal_delay: HashMap<Observable, f64>,
    /// Time Reference Delay, as defined by BIPM in
    /// "GPS Receivers Accurate Time Comparison" : the time delay
    /// between the GNSS receiver external reference clock and the sampling clock
    /// (once again can be persued as a cable delay). This one is typically
    /// only required in ultra high precision timing applications
    pub time_ref_delay: Option<f64>,
    /// Minimal percentage ]0; 1[ of Sun light to be received by an SV
    /// for not to be considered in Eclipse.
    /// A value closer to 0 means we tolerate fast Eclipse exit.
    /// A value closer to 1 is a stringent criteria: eclipse must be totally exited.
    #[cfg_attr(feature = "serde", serde(default))]
    pub min_sv_sunlight_rate: Option<f64>,
    /// Minimal elevation angle. SV below that angle will not be considered.
    #[cfg_attr(feature = "serde", serde(default))]
    pub min_sv_elev: Option<f64>,
    /// Minimal SNR for an SV to be considered.
    #[cfg_attr(feature = "serde", serde(default))]
    pub min_sv_snr: Option<SNR>,
    /// modeling
    #[cfg_attr(feature = "serde", serde(default))]
    pub modeling: Modeling,
    /// Max. number of vehicules to consider.
    /// The more the merrier, but it also means heavier computations
    #[cfg_attr(feature = "serde", serde(default = "default_max_sv"))]
    pub max_sv: usize,
}

impl RTKConfig {
    pub fn default(solver: SolverType) -> Self {
        match solver {
            SolverType::SPP => Self {
                timescale: default_timescale(),
                fixed_altitude: None,
                interp_order: default_interp(),
                code_smoothing: default_smoothing(),
                min_sv_sunlight_rate: None,
                min_sv_elev: Some(10.0),
                min_sv_snr: Some(SNR::from_str("weak").unwrap()),
                modeling: Modeling::default(),
                max_sv: default_max_sv(),
                internal_delay: Default::default(),
                time_ref_delay: Default::default(),
            },
            SolverType::PPP => Self {
                timescale: default_timescale(),
                fixed_altitude: None,
                interp_order: 11,
                code_smoothing: default_smoothing(),
                min_sv_sunlight_rate: Some(0.75),
                min_sv_elev: Some(25.0),
                min_sv_snr: Some(SNR::from_str("strong").unwrap()),
                modeling: Modeling::default(),
                max_sv: default_max_sv(),
                internal_delay: Default::default(),
                time_ref_delay: Default::default(),
            },
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum SolverMode {
    /// Receiver is kept at fixed location
    #[default]
    Static,
    /// Receiver is not static
    Kinematic,
}
