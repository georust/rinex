use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    epoch::parse_in_timescale as parse_epoch_in_timescale,
    prelude::{Epoch, ParsingError, TimeScale},
};

/// System Time Offset Message
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StoMessage {
    /// Time System
    pub system: String,
    /// UTC ID
    pub utc: String,
    /// Message transmmission time in seconds of GNSS week
    pub t_tm: u32,
    /// ((s), (s.s⁻¹), (s.s⁻²))
    pub a: (f64, f64, f64),
}

impl StoMessage {
    pub fn parse(
        mut lines: std::str::Lines<'_>,
        ts: TimeScale,
    ) -> Result<(Epoch, Self), ParsingError> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(ParsingError::EmptyEpoch),
        };

        let (epoch, rem) = line.split_at(23);
        let (system, _) = rem.split_at(5);
        let epoch = parse_epoch_in_timescale(epoch.trim(), ts)?;

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(ParsingError::EmptyEpoch),
        };

        let (time, rem) = line.split_at(23);
        let (a0, rem) = rem.split_at(19);
        let (a1, rem) = rem.split_at(19);
        let (a2, rem) = rem.split_at(19);

        let t_tm = f64::from_str(time.trim()).map_err(|_| ParsingError::SystemTimeData)?;

        Ok((
            epoch,
            Self {
                system: system.trim().to_string(),
                t_tm: t_tm as u32,
                a: (
                    f64::from_str(a0.trim()).map_err(|_| ParsingError::SystemTimeData)?,
                    f64::from_str(a1.trim()).map_err(|_| ParsingError::SystemTimeData)?,
                    f64::from_str(a2.trim()).map_err(|_| ParsingError::SystemTimeData)?,
                ),
                utc: rem.trim().to_string(),
            },
        ))
    }
}
