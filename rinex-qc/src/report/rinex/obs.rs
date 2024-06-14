use crate::report::shared::SamplingReport;

/// Frequency dependent pagination
pub struct FrequencyPage {
    /// Carrier
    pub carrier: Carrier,
    /// Loss of sight analysis
    pub gaps: HashMap<Epoch, Duration>,
    /// Loss of lock analysis
    pub lockloss: HashMap<Epoch, LockLossEvent>,
    /// Cycle slip analysis
    pub cs: HashMap<Epoch, CsSource>,
}

/// Constellation dependent pagination
pub struct ConstellationPage {
    /// True when doppler are sampled
    pub doppler: bool,
    /// True if Standard Positioning compatible
    pub spp_compatible: bool,
    /// True if Code Dual Frequency Positioning compatible
    pub cpp_compatible: bool,
    /// True if PPP compatible
    pub ppp_compatible: bool,
    /// Frequency dependent pagination
    pub pages: Vec<FrequencyPage>,
}

/// RINEX Observation Report
pub struct Report {
    pub receiver: Receiver,
    /// Time frame and sampling analysis
    pub sampling: SamplingReport,
    pub constellations: Vec<Constellation>,
    /// Constellation dependent pagination
    pub pages: HashMap<Constellation, ConstellationPage>,
}
