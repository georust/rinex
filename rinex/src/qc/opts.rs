use crate::prelude::*;

#[cfg(feature = "serde")]
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Serialize, Serializer, Deserialize, Deserializer, de::Error};

#[derive(Clone, Debug)]
pub enum SlotError {
	ParsingError
}

#[derive(Debug, Clone, PartialEq)]
pub enum Slot {
	Duration(Duration),
	Percentage(f64),
}

impl std::str::FromStr for Slot {
	type Err = SlotError;
	fn from_str(content: &str) -> Result<Self, Self::Err> {
		let c = content.trim();
		if let Ok(dt) = Duration::from_str(c) {
			Ok(Self::Duration(dt))
		} else if let Ok(f) = f64::from_str(c) {
			Ok(Self::Percentage(f))
		} else {
			Err(SlotError::ParsingError)
		}
	}
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Slot {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> 
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		if let Ok(dt) = Duration::from_str(s.trim()) {
			Ok(Self::Duration(dt))
		} else {
			let f = f64::from_str(&s.replace("%", "").trim()).map_err(D::Error::custom)?;
			Ok(Self::Percentage(f))
		}
	}
}
/*
	fn serialize<S>(slot: Slot, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let s = match slot {
			Slot::Percentage(f) => f.to_string(),
			Slot::Duration(dt) => dt.to_string(),
		};
		serializer.serialize_str(&s)
	}
*/

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum CsStrategy {
	/// Study CS events and report them
	Study,
	/// Study CS events and repair them
	StudyAndRepair,
}

impl Default for CsStrategy {
	fn default() -> Self {
		Self::Study
	}
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct StatisticsOpts {
	/// Window/slot duration, on which we evaluate all statistics
	pub window: Slot,
}

impl Default for StatisticsOpts {
	fn default() -> Self {
		Self {
			window: Slot::Percentage(25.0),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct ProcessingOpts {
	/// Cs analysis/reporting strategy
	pub cs: CsStrategy,
	/// Ionospheric variation tolerance
	pub iono_rate_tolerance: f64,
	pub iono_rate_tolerance_dt: Duration,
	/// Clock Drift Moving average window slot
	pub clock_drift_window: Duration, 
	/// Increment of the elelavtion mask
	pub elev_mask_increment: f64,
}

impl Default for ProcessingOpts {
	fn default() -> Self {
		Self {
			cs: CsStrategy::default(),
			iono_rate_tolerance: 400.0E-2_f64,
			iono_rate_tolerance_dt: Duration::from_seconds(60.0_f64),
			clock_drift_window: Duration::from_seconds(600.0_f64),
			elev_mask_increment: 10.0_f64,
		}
	}
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct GroundPosition {
	pub ecef: (f64,f64,f64),
	pub geo: (f64,f64,f64),
}

impl GroundPosition {
	fn from_ecef_wgs84(pos: (f64,f64,f64)) -> Self {
		Self {
			ecef: pos.clone(),
			geo: map_3d::ecef2geodetic(pos.0, pos.1, pos.2, map_3d::Ellipsoid::WGS84)
		}
	}
	fn from_geo_wgs84(pos: (f64,f64,f64)) -> Self {
		Self {
			geo: pos.clone(),
			ecef: map_3d::geodetic2ecef(pos.0, pos.1, pos.2, map_3d::Ellipsoid::WGS84)
		}
	}
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct QcOpts {
	/// Custom duration considered as a data gap
    pub manual_gap: Option<Duration>,
	/// Manually defined Ground position (ECEF)
	pub ground_position: Option<GroundPosition>,
}

impl QcOpts {
	pub fn with_ground_position_ecef(&self, pos: (f64,f64,f64)) -> Self {
		let mut s = self.clone();
		s.ground_position = Some(GroundPosition::from_ecef_wgs84(pos));
		s
	}
	pub fn with_ground_position_geo(&self, pos: (f64,f64,f64)) -> Self {
		let mut s = self.clone();
		s.ground_position = Some(GroundPosition::from_geo_wgs84(pos));
		s
	}
}

impl Default for QcOpts {
    fn default() -> Self {
        Self {
            manual_gap: None,
			ground_position: None,
        }
    }
}

#[cfg(feature = "serde")]
#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn qc_opts_serdes() {
		let content = r#"
			{
				"ground_pos_ecef": "(1.0, 2.0, 3.0)"
			}"#;
		let opts: QcOpts = serde_json::from_str(content).unwrap();
		assert!(opts.manual_gap.is_none());
		
		/*let content = r#"
			{
				"reporting": "PerSv",
				"statistics": {
					"window": "10 seconds"
				}
			}"#;

		let opts: QcOpts = serde_json::from_str(content).unwrap();
		assert_eq!(opts.reporting, ReportingStrategy::PerSv);
		assert_eq!(opts.statistics, Some(StatisticsOpts {
			window: Slot::Duration(Duration::from_seconds(10.0)),
		}));
		assert!(opts.processing.is_none());
		
		let content = r#"
			{
				"reporting": "PerSignal",
				"statistics": {
					"window": "10 %"
				}
			}"#;

		let opts: QcOpts = serde_json::from_str(content).unwrap();
		assert_eq!(opts.reporting, ReportingStrategy::PerSignal);
		assert_eq!(opts.statistics, Some(StatisticsOpts {
			window: Slot::Percentage(10.0_f64),
		}));
		assert!(opts.processing.is_none());
		*/
	}
}
