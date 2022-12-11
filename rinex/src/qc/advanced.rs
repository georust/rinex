#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ElMaskable {
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
    pub elevation_mask: ElMaskable,
}
