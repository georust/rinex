use crate::model::Modeling;
use crate::SolverType;
use hifitime::prelude::TimeScale;

#[cfg(feature = "serde")]
use serde::Deserialize;

use rinex::prelude::{Constellation, GroundPosition};

fn default_timescale() -> TimeScale {
    TimeScale::GPST
}

fn default_gnss() -> Vec<Constellation> {
    vec![Constellation::GPS]
}

fn default_interp() -> usize {
    7
}

fn default_max_sv() -> usize {
    10
}

fn default_tgd() -> bool {
    true
}

fn default_smoothing() -> bool {
    false
}

fn default_iono() -> bool {
    false
}

fn default_tropo() -> bool {
    false
}

#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct RTKConfig {
    /// Time scale
    #[cfg_attr(feature = "serde", serde(default = "default_timescale"))]
    pub timescale: TimeScale,
    /// positioning mode
    #[cfg_attr(feature = "serde", serde(default))]
    pub mode: SolverMode,
    /// (Position) interpolation filter order.
    /// A minimal order must be respected for correct results.
    /// -  7 when working with broadcast ephemeris
    /// - 11 when working with SP3
    #[cfg_attr(feature = "serde", serde(default = "default_interp"))]
    pub interp_order: usize,
    /// Whether the solver is working in fixed altitude mode or not
    #[cfg_attr(feature = "serde", serde(default))]
    pub fixed_altitude: Option<f64>,
    /// Position receveir position, if known before hand
    pub rcvr_position: Option<GroundPosition>,
    /// constellation to consider,
    #[cfg_attr(feature = "serde", serde(default = "default_gnss"))]
    pub gnss: Vec<Constellation>,
    /// PR code smoothing filter before moving forward
    #[cfg_attr(feature = "serde", serde(default = "default_smoothing"))]
    pub code_smoothing: bool,
    /// true if we're using troposphere modeling
    #[cfg_attr(feature = "serde", serde(default = "default_tropo"))]
    pub tropo: bool,
    /// true if we're using ionosphere modeling
    #[cfg_attr(feature = "serde", serde(default = "default_iono"))]
    pub iono: bool,
    /// true if we're using total group delay modeling
    #[cfg_attr(feature = "serde", serde(default = "default_tgd"))]
    pub tgd: bool,
    /// Minimal percentage ]0; 1[ of Sun light to be received by an SV
    /// for not to be considered in Eclipse.
    /// A value closer to 0 means we tolerate fast Eclipse exit.
    /// A value closer to 1 is a stringent criteria: eclipse must be totally exited.
    #[cfg_attr(feature = "serde", serde(default))]
    pub min_sv_sunlight_rate: Option<f64>,
    /// modeling
    #[cfg_attr(feature = "serde", serde(default))]
    pub modeling: Modeling,
    /// max. vehicules supported,
    /// the more the merrier, but heavier computations
    #[cfg_attr(feature = "serde", serde(default = "default_max_sv"))]
    pub max_sv: usize,
}

impl RTKConfig {
    pub fn default(solver: SolverType) -> Self {
        match solver {
            SolverType::SPP => Self {
                timescale: default_timescale(),
                mode: SolverMode::default(),
                gnss: default_gnss(),
                fixed_altitude: None,
                rcvr_position: None,
                interp_order: default_interp(),
                code_smoothing: default_smoothing(),
                tropo: default_tropo(),
                iono: default_iono(),
                tgd: default_tgd(),
                min_sv_sunlight_rate: None,
                modeling: Modeling::default(),
                max_sv: default_max_sv(),
            },
            SolverType::PPP => Self {
                timescale: default_timescale(),
                mode: SolverMode::default(),
                gnss: default_gnss(),
                fixed_altitude: None,
                rcvr_position: None,
                interp_order: 11,
                code_smoothing: default_smoothing(),
                tropo: default_tropo(),
                iono: default_iono(),
                tgd: default_tgd(),
                min_sv_sunlight_rate: Some(0.75),
                modeling: Modeling::default(),
                max_sv: default_max_sv(),
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
