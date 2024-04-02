use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(docrs)]
pub use bibliography::Bibliography;

/// DOMES parsing error
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid domes format")]
    InvalidFormat,
    #[error("invalid domes length")]
    InvalidLength,
}

/// DOMES site reference point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TrackingPoint {
    /// Monument (pole, pillar, geodetic marker..)
    Monument,
    /// Instrument reference point.
    /// This is usually the antenna reference point, but it can be any
    /// location referred to an instrument, like a specific location
    /// on one axis of a telescope.
    Instrument,
}

/// DOMES Site identifier, see [Bibliography::IgnItrfDomes]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Domes {
    /// Area / Country code (3 digits)
    pub area: u16,
    /// Area site number (2 digits)
    pub site: u8,
    /// Tracking point
    pub point: TrackingPoint,
    /// Sequential number (3 digits)
    pub sequential: u16,
}

impl std::str::FromStr for Domes {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 9 {
            let point = if s[5..6].eq("M") {
                TrackingPoint::Monument
            } else if s[5..6].eq("S") {
                TrackingPoint::Instrument
            } else {
                return Err(Error::InvalidFormat);
            };
            let area = s[..3].parse::<u16>().map_err(|_| Error::InvalidFormat)?;
            let site = s[3..5].parse::<u8>().map_err(|_| Error::InvalidFormat)?;
            let sequential = s[6..].parse::<u16>().map_err(|_| Error::InvalidFormat)?;
            Ok(Self {
                point,
                area,
                site,
                sequential,
            })
        } else {
            Err(Error::InvalidLength)
        }
    }
}

impl std::fmt::Display for Domes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let point = match self.point {
            TrackingPoint::Monument => 'M',
            TrackingPoint::Instrument => 'S',
        };
        write!(
            f,
            "{:03}{:02}{}{:03}",
            self.area, self.site, point, self.sequential
        )
    }
}

#[cfg(test)]
mod test {
    use super::{Domes, TrackingPoint};
    use std::str::FromStr;
    #[test]
    fn parser() {
        for (descriptor, expected) in [
            (
                "10002M006",
                Domes {
                    area: 100,
                    site: 2,
                    sequential: 6,
                    point: TrackingPoint::Monument,
                },
            ),
            (
                "40405S031",
                Domes {
                    area: 404,
                    site: 5,
                    sequential: 31,
                    point: TrackingPoint::Instrument,
                },
            ),
        ] {
            let domes = Domes::from_str(descriptor).unwrap();
            assert_eq!(domes, expected, "failed to parse DOMES");
            // reciprocal
            assert_eq!(domes.to_string(), descriptor, "DOMES reciprocal failed");
        }
    }
}
