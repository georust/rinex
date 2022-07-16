use thiserror::Error;
use std::collections::HashMap;
use rinex::constellation::Constellation;

#[derive(Debug, PartialEq, Clone)]
/// Describes how the included GNSS
/// bias values have to be interpreted and applied
pub enum BiasMode {
    Relative,
    Absolute,
}

#[derive(Debug, Error)]
pub enum BiasModeError {
    #[error("unknown BiasMode")]
    UnknownBiasMode,
}

impl Default for BiasMode {
    fn default() -> Self {
        Self::Absolute
    }
}

impl std::str::FromStr for BiasMode {
    type Err = BiasModeError;
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        if content.eq("R") {
            Ok(BiasMode::Relative)
        } else if content.eq("A") {
            Ok(BiasMode::Absolute)
        } else {
            Err(BiasModeError::UnknownBiasMode)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TimeSystem {
    /// Time system of given GNSS constellation
    GNSS(Constellation),
    /// Coordinates Universal Time
    UTC,
    /// International Atomic Time
    TAI,
}

impl Default for TimeSystem {
    fn default() -> Self {
        Self::UTC
    }
}

#[derive(Debug)]
pub enum DeterminationMethodError {
    UnknownMethod(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum DeterminationMethod {
    /// Intra Frequency Bias estimation,
    /// is the analysis of differences between
    /// frequencies relying on a ionosphere
    /// reduction model
    IntraFrequencyEstimation,
    /// Inter Frequency Bias estimation,
    /// is the analysis of differences between
    /// observables of different frequencyes,
    /// relying on a ionosphere reduction model
    InterFrequencyEstimation,
    /// Analyzing the ionosphere free linear combination
    ClockAnalysis,
    /// Analyzing the geometry free linear combination
    IonosphereAnalysis,
    /// Results from Clock and Ionosphere analysis combination
    CombinedAnalysis,
}

impl std::str::FromStr for DeterminationMethod {
    type Err = DeterminationMethodError;
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        if content.eq("CLOCK_ANALYSIS") {
            Ok(Self::ClockAnalysis)
        } else if content.eq("INTRA-FREQUENCY_BIAS_ESTIMATION") {
            Ok(Self::IntraFrequencyEstimation)
        } else if content.eq("INTER-FREQUENCY_BIAS_ESTIMATION") {
            Ok(Self::InterFrequencyEstimation)
        } else if content.eq("IONOSPHERE_ANALYSIS") {
            Ok(Self::IonosphereAnalysis)
        } else if content.eq("COMBINED_ANALYSIS") {
            Ok(Self::CombinedAnalysis)
        } else {
            Err(DeterminationMethodError::UnknownMethod(content.to_string()))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Description {
    /// Observation Sampling: sampling interval [s]
    /// used for data analysis
    pub sampling: Option<u32>,
    /// Parameter Spacing: spacing interval [s]
    /// used for parameter representation
    pub spacing: Option<u32>,
    /// Method used to generate the bias results
    pub method: Option<DeterminationMethod>,
    /// See [BiasMode]
    pub bias_mode: BiasMode,
    /// TimeSystem, see [TimeSystem]
    pub system: TimeSystem,
    /// Receiver clock reference GNSS
    pub rcvr_clk_ref: Option<Constellation>,
    /// Satellite clock reference observables:
    /// list of observable codes (standard 3 letter codes),
    /// for each GNSS in this file.
    /// Must be provided if associated bias results are consistent
    /// with the ionosphere free LC, otherwise, these might be missing
    pub sat_clk_ref: HashMap<Constellation, Vec<String>>
}

impl Default for Description {
    fn default() -> Self {
        Self {
            sampling: None,
            spacing: None,
            method: None,
            bias_mode: BiasMode::default(),
            system: TimeSystem::default(),
            rcvr_clk_ref: None,
            sat_clk_ref: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Solution {

}

/*
#[derive(Debug, Clone)]
//#[derive(StrumString)]
pub enum BiasType {
    /// Differential Signal Bias (DSB)
    DSB,
    /// Ionosphere Free Signal bias (ISB)
    ISB,
    /// Observable Specific Signal bias (OSB)
    OBS,
}

pub struct Bias {
    pub btype: BiasType,
    pub sv: rinex::sv::Sv,
    pub station: String,
    pub obs_codes: (String, String),
    pub start_time: chrono::NaiveDateTime,
    pub end_time: chrono::NaiveDateTime,
    pub unit: String,
}*/

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_determination_methods() {
        let method = DeterminationMethod::from_str("COMBINED_ANALYSIS");
        assert_eq!(method.is_ok(), true);
        assert_eq!(method.unwrap(), DeterminationMethod::CombinedAnalysis);
    }
}
