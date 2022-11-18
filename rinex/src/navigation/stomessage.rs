use thiserror::Error;
use std::str::FromStr;
use crate::epoch;
use hifitime::Epoch;

/// Parsing error
#[derive(Debug, Error)]
pub enum Error {
	#[error("missing data")]
	MissingData,
	#[error("failed to parse epoch")]
	EpochError(#[from] epoch::Error),
	#[error("failed to parse data")]
	ParseFloatError(#[from] std::num::ParseFloatError),
}

/// System Time Message 
/// ```
/// use rinex::prelude::*;
/// use rinex::navigation::*;
/// let rnx = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
///     .unwrap();
/// let record = rnx.record.as_nav()
///     .unwrap();
/// for (epoch, classes) in record {
///     for (class, frames) in classes {
///         // epochs may contain other frame classes
///         if *class == FrameClass::SystemTimeOffset {
///             for fr in frames {
///                 let (msg_type, sv, sto) = fr.as_sto()
///                     .unwrap(); // you're fine at this point
///                 let system = &sto.system;
///                 let utc = &sto.utc; // UTC provider
///                 let t_tm = sto.t_tm;
///                 let (a, dadt, ddadt) = sto.a;
///             }
///         }
///     }
/// }
/// ```
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
		let (epoch, _) = epoch::parse(epoch.trim())?;
		
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
