use crate::prelude::*;
use ndarray::{Array4, Array2};

#[derive(Clone, Debug)]
pub enum ConversionError {
    UnexpectedType,
}

/// Conversion item, for RINEX <=> NDarray conversions,
/// for eased up calculations
#[derive(Clone, PartialEq)]
pub enum CvItem {
    Epoch(Epoch),
    Sv(Sv),
    Observable(Observable),
    Orbit(String),
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
            Self::Orbit(s) => Self::Orbit(s),
            Self::Data(f) => Self::Data(f),
            Self::Observable(o) => Self::Observable(o),
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

impl From<Observable> for CvItem {
    fn from(obs: Observable) -> Self {
        Self::Observable(obs)
    }
}

impl From<&str> for CvItem {
    fn from(orb: &str) -> Self {
        Self::Orbit(orb.to_string())
    }
}

impl From<f64> for CvItem {
    fn from(data: f64) -> Self {
        Self::Data(data)
    }
}

/// Conversion trait for algebric and statistical opreations
pub trait Conversion {
    fn to_ndarray(&self) -> Array2<CvItem>;
    fn from_ndarray(&self, arr: Array4<CvItem>) -> Result<Self, ConversionError> where Self: Sized;
}
