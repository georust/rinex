use thiserror::Error;

use crate::{
    error::ParsingError,
    prelude::{Constellation, Epoch},
};

#[derive(Debug, Clone)]
pub struct Receiver {
    /// Station name
    pub station: String,
    /// Receiver constellation dependence
    pub constellation: Option<Constellation>,
    /// Receiver group name
    pub group: String,
    /// Receiver validity
    pub valid_from: Epoch,
    /// Receiver validity
    pub valid_until: Epoch,
    /// Receiver type
    pub rtype: String,
    /// Firmware descriptor
    pub firmware: String,
}

impl std::str::FromStr for Receiver {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (station, rem) = s.split_at(10);
        let (constellation, rem) = rem.split_at(2);
        let (group, rem) = rem.split_at(10);
        let (start, rem) = rem.split_at(15);
        let (end, rem) = rem.split_at(15);
        let (rtype, rem) = rem.split_at(21);
        Ok(Receiver {
            station: station.trim().to_string(),
            constellation: {
                if let Ok(c) = Constellation::from_str(constellation.trim()) {
                    Some(c)
                } else {
                    None
                }
            },
            group: group.trim().to_string(),
            valid_from: parse_datetime(start.trim())?,
            valid_until: parse_datetime(end.trim())?,
            rtype: rtype.trim().to_string(),
            firmware: rem.trim().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_receiver() {
        //"STATION__ C GROUP____ DATA_START____ DATA_END______ RECEIVER_TYPE_______ RECEIVER_FIRMWARE___"
        let rcvr = Receiver::from_str(
            "MAO0      G @MP0      2015:276:00000 2015:276:86399 JAVAD TRE-G3TH DELTA 3.6.4",
        );
        assert!(rcvr.is_ok());
        let rcvr = rcvr.unwrap();
        assert_eq!(rcvr.station, "MAO0");
        assert_eq!(rcvr.group, "@MP0");
        assert_eq!(rcvr.firmware, "3.6.4");
        assert_eq!(rcvr.rtype, "JAVAD TRE-G3TH DELTA");
    }
}
