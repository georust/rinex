use strum_macros::EnumString;
use super::{
    prelude::*,
    navigation::*,
    observation::*,
};
use std::collections::{HashMap, BTreeMap};

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Grade {
    #[strum(serialize = "A++")] 
    GradeApp,
    #[strum(serialize = "A+")] 
    GradeAp,
    #[strum(serialize = "A")] 
    GradeA,
    #[strum(serialize = "B")]
    GradeB,
    #[strum(serialize = "C")]
    GradeC,
    #[strum(serialize = "D")]
    GradeD,
    #[strum(serialize = "E")]
    GradeE,
    #[strum(serialize = "F")]
    GradeF,
}

/// Sampling QC report
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SamplingQc {
    /// First epoch identified
    pub first_epoch: Epoch,
    /// Last epoch identified
    pub last_epoch: Epoch,
    /// Time line
    pub time_line: Duration,
    /// Unusual data gaps
    pub gaps: Vec<(Epoch, Epoch)>
}

impl SamplingQc {
    pub fn new(rnx: &Rinex) -> Self {
        let first_epoch = rnx.first_epoch()
            .expect("Sampling QC expects a RINEX classed by epoch");
        let last_epoch = rnx.last_epoch()
            .expect("Sampling QC expects a RINEX classed by epoch");
        Self {
            first_epoch,
            last_epoch,
            time_line: last_epoch - first_epoch,
            gaps: rnx.gaps_analysis(),
        }
    }
}

/// Observation RINEX specific QC report
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ObservationQc {
    /// Total epochs with at least 1 Observation
    pub epochs_with_obs: usize,
    /// Total Sv with at least 1 Observation
    pub total_sv_with_obs: usize,
    /// Total Sv without a single Observation
    pub total_sv_without_obs: usize,
    /// Total Receiver Clock Offsets
    pub total_clk: usize,
    /// Receiver reset events and related timestamps 
    pub rcvr_resets: Vec<(Epoch, Epoch)>,
    /// Average received power per Sv
    pub mean_s: HashMap<Sv, f64>,
    /// Phase Differential Code biases
    pub dcbs: HashMap<String, HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>,
}

/// Navigation RINEX specific QC report
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NavigationQc {
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdvancedElMaskable {
    pub possible_obs: usize,
    pub complete_obs: usize,
    pub deleted_obs: usize,
    pub masked_obs: usize,
}

/// Advanced QC report, generated
/// when both Observation and Navigation data was provided
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdvancedQc {
    pub elevation_mask: AdvancedElMaskable,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QcReport {
    /// Sampling QC
    pub sampling: SamplingQc,
    /// Observation RINEX specific QC
    pub observation: Option<ObservationQc>, 
    /// Navigation RINEX specific QC
    pub navigation: Option<NavigationQc>,
    /// Advanced Observation + Navigation specific QC
    pub advanced: Option<AdvancedQc>,
}

pub struct QcOpts {
    pub el_mask: f64,
}

impl Default for QcOpts {
    fn default() -> Self {
        Self {
            el_mask: 10.0_f64,
        }
    }
}

impl QcReport {
    /// Processes given RINEX and generates
    /// a QC report
    fn new(rnx: &Rinex, Opts: QcOpts) -> Self {
        Self {
            sampling: SamplingQc::new(rnx),
            observation: None,
            navigation: None,
            advanced: None,
        }
    }
}
