use crate::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Slot {
	Duration(Duration),
	Percentage(f64),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ReportingStrategy {
	/// Report everything
	All,
	/// Report on a Sv basis
	PerSv,
	/// Report on a signal basis
	PerSignal,
	/// Report on a GNSS system basis (default)
	PerConstellation,
}

impl Default for ReportingStrategy {
	fn default() -> Self {
		Self::PerConstellation
	}
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
			cs: CsStrategy,
			iono_rate_tolerance: 400.0E-2_f64,
			iono_rate_tolerance_dt: Duration::from_minutes(1.0_f64),
			clock_drift_window: Duration::from_minutes(10.0_f64),
			elev_mask_increment: 10.0_f64,
		}
	}
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QcOpts {
	/// Reporting strategy
	pub reporting: ReportingStrategy,
	/// Statiscal analysis
	pub statistics: Option<StatisticsOpts>,
	/// Processing opts
	pub processing: Option<ProcessingOpts>,
	/// Custom duration considered as a data gap
    pub manual_gap: Option<Duration>,
}

impl Default for QcOpts {
    fn default() -> Self {
        Self {
            manual_gap: None,
			reporting: ReportStrategy::default(),
            statitics: StatisticsOpts::default(), 
			processing: ProcessingOpts::default(),
        }
    }
}
