use thiserror::Error;
use std::str::FromStr;
use crate::{
	Epoch, EpochFlag, 
	epoch::str2date, epoch::ParseDateError,
};

/// Parsing error
#[derive(Debug, Error)]
pub enum Error {
	#[error("missing data")]
	MissingData,
	#[error("failed to parse date")]
	ParseDateError(#[from] ParseDateError),
	#[error("failed to parse data")]
	ParseFloatError(#[from] std::num::ParseFloatError),
}

/// System Time Message 
#[derive(Debug, Clone)]
#[derive(Default)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct StoMessage {
    /// Time System
    pub system: String,
    /// UTC ID
    pub utc: String,
    /// Message transmmission time [s] of GNSS week
    pub t_tm: u32,
    /// ([sec], [sec.sec⁻¹], [sec.sec⁻²])
    pub a: (f64,f64,f64),
}

impl StoMessage {
    pub fn parse (mut lines: std::str::Lines<'_>) -> Result<(Epoch, Self), Error> {
		let line = match lines.next() {
			Some(l) => l,
			_ => return Err(Error::MissingData),
		};
		
		let (epoch, rem) = line.split_at(23);
		let (system, _) = rem.split_at(5);
		let epoch = Epoch {
			date: str2date(epoch.trim())?,
			flag: EpochFlag::Ok,
		};
		
		let line = match lines.next() {
			Some(l) => l,
			_ => return Err(Error::MissingData),
		};
		let (time, rem) = line.split_at(23);
		let (a0, rem) = rem.split_at(19);
		let (a1, rem) = rem.split_at(19);
		let (a2, rem) = rem.split_at(19);

		let t_tm = f64::from_str(time.trim())?;
		Ok((epoch,
			Self {
				system: system.trim().to_string(),
				t_tm: t_tm as u32,
				a: (
					f64::from_str(a0.trim()).unwrap_or(0.0_f64),
					f64::from_str(a1.trim()).unwrap_or(0.0_f64),
					f64::from_str(a2.trim()).unwrap_or(0.0_f64),
				),
				utc: rem.trim().to_string(),
			},
		))
	}
}
