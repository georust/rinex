use std::str::FromStr;
use thiserror::Error;
use rinex::constellation::Constellation;

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
}
