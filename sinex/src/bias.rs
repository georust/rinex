use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
/// Bias mode description, for Header field
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

/*pub struct Header {
    pub input: String,
    pub output: String,
    pub contact: String,
    pub hardware: String,
    pub software: String,
    pub reference_frame: String,
}

pub enum DataType {
    ObsSampling,
    ParmeterSpacing,
    DeterminationMethod,
    BiasMode,
    TimeSystem,
    ReceiverClockRef,
    SatelliteClockReferenceObs,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_receiver() {
        //"STATION__ C GROUP____ DATA_START____ DATA_END______ RECEIVER_TYPE_______ RECEIVER_FIRMWARE___"
        let rcvr = Receiver::from_str(
        "MAO0      G @MP0      2015:276:00000 2015:276:86399 JAVAD TRE-G3TH DELTA 3.6.4");
        assert_eq!(rcvr.is_ok(), true);
        let rcvr = rcvr.unwrap();
        println!("{:?}", rcvr);
        assert_eq!(rcvr.station, "MAO0");
        assert_eq!(rcvr.group, "@MP0");
        assert_eq!(rcvr.firmware, "3.6.4");
        assert_eq!(rcvr.rtype, "JAVAD TRE-G3TH DELTA");
    }
}

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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_receiver() {
        //"STATION__ C GROUP____ DATA_START____ DATA_END______ RECEIVER_TYPE_______ RECEIVER_FIRMWARE___"
        let rcvr = Receiver::from_str(
        "MAO0      G @MP0      2015:276:00000 2015:276:86399 JAVAD TRE-G3TH DELTA 3.6.4");
        assert_eq!(rcvr.is_ok(), true);
        let rcvr = rcvr.unwrap();
        println!("{:?}", rcvr);
        assert_eq!(rcvr.station, "MAO0");
        assert_eq!(rcvr.group, "@MP0");
        assert_eq!(rcvr.firmware, "3.6.4");
        assert_eq!(rcvr.rtype, "JAVAD TRE-G3TH DELTA");
    }
}*/
