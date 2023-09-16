use crate::solver::SolverType;
use rinex::prelude::Constellation;
use rinex::prelude::GroundPosition;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct SolverOpts {
    /// Criteria (for convergence)
    pub epsilon: f64,
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
    pub rcvr_position: GroundPosition,
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
}

impl SolverOpts {
    pub fn default(solver: SolverType) -> Self {
        match solver {
            SolverType::SPP => Self {
                epsilon: 5.0_f64,
                gnss: vec![Constellation::GPS, Constellation::Galileo],
                fixed_altitude: None,
                rcvr_position: GroundPosition::default(),
                interp_order: 7,
                positioning: PositioningMode::default(),
                code_smoothing: false,
                tropo: false,
                iono: false,
                tgd: false,
            },
            SolverType::PPP => Self {
                epsilon: 0.1_f64,
                gnss: vec![Constellation::GPS, Constellation::Galileo],
                fixed_altitude: None,
                rcvr_position: GroundPosition::default(),
                interp_order: 11,
                positioning: PositioningMode::default(),
                code_smoothing: false,
                tropo: false,
                iono: false,
                tgd: false,
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
pub enum SpecificOpts {
    /// SPP solver specific parameters
    SPPSpecificOpts(SppOpts),
    /// PPP solver specific parameters
    PPPSpecificOpts(PppOpts),
}

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
