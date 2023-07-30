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
    /// Grid start coordinates in decimal degrees
    pub start: f64,
    /// Grid end coordinates in decimal degrees
    pub end: f64,
    /// Grid spacing (inncrement value), in decimal degrees
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
    // Returns grid length, in terms of data points
    pub fn length(&self) -> usize {
        (self.end / self.spacing).floor() as usize
    }
    /// Returns true if self is a single point space
    pub fn is_single_point(&self) -> bool {
        (self.end == self.start) && self.spacing == 0.0
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

/// Reference Grid,
/// defined in terms of Latitude, Longitude and Altitude.
/// If 2D-TEC maps, static altitude is defined, ie.:
/// start = end altitude and spacing = 0.
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Grid {
    /// Latitude
    pub latitude: GridLinspace,
    /// Longitude
    pub longitude: GridLinspace,
    /// Altitude
    pub height: GridLinspace,
}

impl Grid {
    /// Returns true if self is defined for 3D TEC map
    pub fn is_3d_grid(&self) -> bool {
        !self.is_2d_grid()
    }
    /// Returns true if self is defined to 2D TEC maps,
    /// ie.: static altitude ref point with no altitude space
    /// definition.
    pub fn is_2d_grid(&self) -> bool {
        self.height.is_single_point()
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
        let grid = GridLinspace::new(1.0, 10.0, 1.0).unwrap();
        assert_eq!(grid.length(), 10);
        assert_eq!(grid.is_single_point(), false);
    }
}
