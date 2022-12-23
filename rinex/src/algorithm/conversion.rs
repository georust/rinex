use crate::prelude::*;
use crate::observation::ObservationData;
use ndarray::{Array4, Array3, Array2, Array1, ShapeError};

#[derive(Clone, Debug)]
pub enum ConversionError {
    UnexpectedType,
}

/// Conversion item, for RINEX / ndarray conversions
#[derive(Debug, Clone, PartialEq)]
pub enum CvItem {
    Epoch(Epoch),
    Sv(Sv),
    Observation((Observable, ObservationData)),
    Data(f64),
}

impl CvItem {
    fn as_epoch(&self) -> Option<Epoch> {
        match self {
            Self::Epoch(e) => Some(*e),
            _ => None,
        }
    }
}

impl num::Zero for CvItem {
    fn zero() -> Self {
        Self::Data(0_f64)
    }
    fn is_zero(&self) -> bool {
        *self == Self::Data(0_f64)
    }
}

impl std::ops::Add for CvItem {
    type Output = Self;    
    fn add(self, rhs: Self) -> Self {
        match self {
            Self::Epoch(lhs_e) => {
                match rhs {
                    Self::Epoch(rhs_e) => Self::Epoch(
                        Epoch::from_duration(lhs_e.to_utc_duration() + rhs_e.to_utc_duration(), TimeScale::UTC)),
                    _ => Self::Epoch(Epoch::default()),
                }
            },
            Self::Sv(sv) => Self::Sv(sv),
            Self::Observation((lhs_observable, lhs_observation)) => {
                match rhs {
                    Self::Observation((rhs_observable, rhs_observation)) => {
                        if lhs_observable == rhs_observable {
                            Self::Observation((lhs_observable, lhs_observation + rhs_observation))
                        } else {
                            Self::Observation((lhs_observable, lhs_observation))
                        }
                    }
                    _ => Self::Observation((lhs_observable, lhs_observation)),
                }
            },
            Self::Data(lhs) => {
                match rhs {
                    Self::Data(rhs) => Self::Data(lhs + rhs),
                    _ => Self::Data(lhs),
                }
            },
        }
    }
}

impl From<Epoch> for CvItem {
    fn from(e: Epoch) -> Self {
        Self::Epoch(e)
    }
}

impl From<Sv> for CvItem {
    fn from(sv: Sv) -> Self {
        Self::Sv(sv)
    }
}

impl From<(Observable, ObservationData)> for CvItem {
    fn from(data: (Observable, ObservationData)) -> Self {
        Self::Observation(data)
    }
}

impl From<f64> for CvItem {
    fn from(data: f64) -> Self {
        Self::Data(data)
    }
}

/// Conversion trait to convert back & forth to `ndarray`
pub trait Conversion {
    fn to_ndarray(&self) -> Result<Array2<CvItem>, ShapeError>;
    fn from_ndarray(&self, arr: Array4<CvItem>) -> Result<Self, ConversionError> where Self: Sized;
}
