use thiserror::Error;
use strum_macros::EnumString;
use std::collections::HashMap;
use rinex::constellation::Constellation;
use crate::{parse_datetime, ParseDateTimeError};

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

#[derive(Debug, PartialEq, Clone)]
#[derive(EnumString)]
//#[derive(StrumString)]
pub enum BiasType {
    /// Differential Signal Bias (DSB)
    DSB,
    /// Ionosphere Free Signal bias (ISB)
    ISB,
    /// Observable Specific Signal bias (OSB)
    OSB,
}

#[derive(Debug, Error)]
pub enum SolutionParsingError {
    #[error("failed to parse BiasType")]
    ParseBiasTypeError(#[from] strum::ParseError),
    #[error("failed to parse bias estimate")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse datetime field")]
    ParseDateTimeError(#[from] ParseDateTimeError),
}

#[derive(Debug, Clone)]
pub struct Solution {
    /// Bias type
    pub btype: BiasType,
    /// Satellite SVN
    pub svn: String,
    /// Space Vehicule ID
    pub prn: String,
    /// Station codes
    pub station: Option<String>,
    /// Observable codes used for estimating the biases,
    /// notes as (OBS1, OBS2) in standards
    pub obs: (String, Option<String>),
    /// Start time for the bias estimate
    pub start_time: chrono::NaiveDateTime,
    /// End time for the bias estimate
    pub end_time: chrono::NaiveDateTime,
    /// Bias parameter unit
    pub unit: String,
    /// Bias parameter estimate (offset)
    pub estimate: f64,
    /// Bias parameter stddev
    pub stddev: f64,
    /// Bias parameter slope estimate
    pub slope: Option<f64>,
    /// Bias parameter slope stddev estimate
    pub slope_stddev: Option<f64>,
}

impl std::str::FromStr for Solution {
    type Err = SolutionParsingError;
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        let (bias_type, rem) = content.split_at(5);
        let (svn, rem) = rem.split_at(5);
        let (prn, rem) = rem.split_at(4);
        let (station, rem) = rem.split_at(10);
        let (obs1, rem) = rem.split_at(5);
        let (obs2, rem) = rem.split_at(5);
        let (start_time, rem) = rem.split_at(15);
        let (end_time, rem) = rem.split_at(15);
        let (unit, rem) = rem.split_at(5);
        let (estimate, rem) = rem.split_at(22);
        let (stddev, rem) = rem.split_at(12);
        Ok(Solution {
            btype: BiasType::from_str(bias_type.trim())?,
            svn: svn.trim().to_string(),
            prn: prn.trim().to_string(),
            station: {
                if station.trim().len() > 0 {
                    Some(station.trim().to_string())
                } else {
                    None
                }
            },
            unit: unit.trim().to_string(),
            start_time: parse_datetime(start_time.trim())?,
            end_time: parse_datetime(end_time.trim())?,
            obs: {
                if obs2.trim().len() > 0 {
                    (obs1.trim().to_string(), Some(obs2.trim().to_string()))
                } else {
                    (obs1.trim().to_string(), None)
                }
            },
            estimate: f64::from_str(estimate.trim())?,
            stddev: f64::from_str(stddev.trim())?,
            slope: None,
            slope_stddev: None,
        })
    }
}

impl Solution {
    /// Returns duration for this bias solution
    pub fn duration (&self) -> chrono::Duration {
        self.end_time - self.start_time
    }
}

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
    #[test]
    fn test_solution_parser() {
        let solution = Solution::from_str(
            "ISB   G    G   GIEN      C1W  C2W  2011:113:86385 2011:115:00285 ns   0.000000000000000E+00 .000000E+00");
        assert_eq!(solution.is_ok(), true);
        let solution = solution.unwrap();
        assert_eq!(solution.btype, BiasType::ISB);
        assert_eq!(solution.svn, "G");
        assert_eq!(solution.prn, "G");
        assert_eq!(solution.station, Some(String::from("GIEN")));
        assert_eq!(solution.obs, (String::from("C1W"), Some(String::from("C2W"))));
        assert_eq!(solution.estimate, 0.0);
        assert_eq!(solution.stddev, 0.0);
        let solution = Solution::from_str(
            "ISB   E    E   GOUS      C1C  C7Q  2011:113:86385 2011:115:00285 ns   -.101593337222667E+03 .259439E+02");
        assert_eq!(solution.is_ok(), true);
        let solution = solution.unwrap();
        assert_eq!(solution.btype, BiasType::ISB);
        assert_eq!(solution.svn, "E");
        assert_eq!(solution.prn, "E");
        assert_eq!(solution.station, Some(String::from("GOUS")));
        assert_eq!(solution.obs, (String::from("C1C"), Some(String::from("C7Q"))));
        assert!((solution.estimate - -0.101593337222667E3) < 1E-6);
        assert!((solution.stddev - 0.259439E+02) < 1E-6);
        let solution = Solution::from_str(
            "OSB   G063 G01           C1C       2016:296:00000 2016:333:00000 ns                 10.2472      0.0062");
        assert_eq!(solution.is_ok(), true);
        let solution = solution.unwrap();
        assert_eq!(solution.btype, BiasType::OSB);
        assert_eq!(solution.svn, "G063");
        assert_eq!(solution.prn, "G01");
        assert_eq!(solution.station, None);
        assert_eq!(solution.obs, (String::from("C1C"), None));
        assert!((solution.estimate - 10.2472) < 1E-4);
        assert!((solution.stddev - 0.0062E+02) < 1E-4);
    }
}
