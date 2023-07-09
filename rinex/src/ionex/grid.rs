use std::ops::Rem;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::Serialize;

/// Grid definition Error
#[derive(Error, Debug)]
pub enum Error {
    #[error("faulty grid definition: `start` and `end` must be multiples of each other")]
    GridStartEndError,
    #[error("faulty grid definition: `start` and `end` must be multiples of `spacing`")]
    GridSpacingError,
}

/// Grid linear space,
/// starting from `start` ranging to `end` (included)
/// with given spacing, defined in km.
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GridLinspace {
    /// Grid start coordinates [ddeg]
    pub start: f64,
    /// Grid end coordinates [ddeg]
    pub end: f64,
    /// Grid spacing (inncrement value), [ddeg]
    pub spacing: f64,
}

impl GridLinspace {
    /// Builds a new Linspace definition
    pub fn new(start: f64, end: f64, spacing: f64) -> Result<Self, Error> {
        let r = end.rem(start);
        /*
         * End / Start must be multiple of one another
         */
        if r == 0.0 {
            if end.rem(spacing) == 0.0 {
                Ok(Self {
                    start,
                    end,
                    spacing,
                })
            } else {
                Err(Error::GridSpacingError)
            }
        } else {
            Err(Error::GridStartEndError)
        }
    }
    /// Returns grid size, in terms of data points
    pub fn size(&self) -> usize {
        ((self.end - self.start) / self.spacing).ceil() as usize
    }
}

impl From<(f64, f64, f64)> for GridLinspace {
    fn from(tuple: (f64, f64, f64)) -> Self {
        Self {
            start: tuple.0,
            end: tuple.1,
            spacing: tuple.2,
        }
    }
}

/// IONEX grid definition
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Grid {
    /// Latitude grid definition
    pub lat_grid: GridLinspace,
    /// Longitude grid definition
    pub lon_grid: GridLinspace,
    /// Altitude grid definition
    pub h_grid: GridLinspace,
}

impl Grid {
    pub fn size(&self) -> usize {
        self.lat_grid.size() * self.lon_grid.size() * self.h_grid.size()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_grid() {
        let default = GridLinspace::default();
        assert_eq!(
            default,
            GridLinspace {
                start: 0.0,
                end: 0.0,
                spacing: 0.0,
            }
        );
        let grid = GridLinspace::new(-180.0, 180.0, 1.0).unwrap();
        assert_eq!(grid.size(), 360);
    }
}
