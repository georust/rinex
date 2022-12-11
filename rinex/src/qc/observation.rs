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

