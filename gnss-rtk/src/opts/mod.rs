use crate::model::Modeling;
use crate::SolverType;
use hifitime::prelude::TimeScale;

use rinex::prelude::{Constellation, GroundPosition};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct SolverOpts {
    /// Time scale
    pub timescale: TimeScale,
    /// (Position) interpolation filter order.
    /// A minimal order must be respected for correct results.
    /// -  7 when working with broadcast ephemeris
    /// - 11 when working with SP3
    pub interp_order: usize,
    /// positioning mode
    pub positioning: PositioningMode,
    /// Whether the solver is working in fixed altitude mode or not
    pub fixed_altitude: Option<f64>,
    /// Position receveir position, if known before hand
    pub rcvr_position: Option<GroundPosition>,
    /// constellation to consider,
    pub gnss: Vec<Constellation>,
    /// PR code smoothing filter before moving forward
    pub code_smoothing: bool,
    /// true if we're using troposphere modeling
    pub tropo: bool,
    /// true if we're using ionosphere modeling
    pub iono: bool,
    /// true if we're using total group delay modeling
    pub tgd: bool,
    /// Minimal percentage ]0; 1[ of Sun light to be received by an SV
    /// for not to be considered in Eclipse.
    /// A value closer to 0 means we tolerate fast Eclipse exit.
    /// A value closer to 1 is a stringent criteria: eclipse must be totally exited.
    pub min_sv_sunlight_rate: Option<f64>,
    /// modeling
    pub modeling: Modeling,
}

impl SolverOpts {
    pub fn default(solver: SolverType) -> Self {
        match solver {
            SolverType::SPP => Self {
                timescale: TimeScale::default(),
                gnss: vec![Constellation::GPS, Constellation::Galileo],
                fixed_altitude: None,
                rcvr_position: None,
                interp_order: 7,
                positioning: PositioningMode::default(),
                code_smoothing: false,
                tropo: false,
                iono: false,
                tgd: false,
                min_sv_sunlight_rate: None,
                modeling: Modeling::default(),
            },
            SolverType::PPP => Self {
                timescale: TimeScale::default(),
                gnss: vec![Constellation::GPS, Constellation::Galileo],
                fixed_altitude: None,
                rcvr_position: None,
                interp_order: 11,
                positioning: PositioningMode::default(),
                code_smoothing: false,
                tropo: false,
                iono: false,
                tgd: false,
                min_sv_sunlight_rate: Some(0.75),
                modeling: Modeling::default(),
            },
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum PositioningMode {
    /// Receiver is kept at fixed location
    #[default]
    Static,
    /// Receiver is not static
    Kinematic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum SpecificOpts {
    /// SPP solver specific parameters
    SPPSpecificOpts(SppOpts),
    /// PPP solver specific parameters
    PPPSpecificOpts(PppOpts),
}

#[allow(dead_code)]
impl SpecificOpts {
    fn spp(&self) -> Option<SppOpts> {
        match self {
            Self::SPPSpecificOpts(opts) => Some(*opts),
            _ => None,
        }
    }
    fn ppp(&self) -> Option<PppOpts> {
        match self {
            Self::PPPSpecificOpts(opts) => Some(*opts),
            _ => None,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct SppOpts {}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct PppOpts {}
