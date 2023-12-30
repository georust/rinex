//! Geodetic marker description
use std::str::FromStr;
use strum_macros::EnumString;

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GeodeticMarker {
    /// Marker name
    pub name: String,
    /// Marker type
    pub marker_type: Option<MarkerType>,
    /// Marker/monument ID
    identification: Option<u32>,
    /// Marker/monument number.
    /// Probably if agency has more than one of them.
    monument: Option<u16>,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MarkerType {
    /// Earth fixed & high precision
    #[strum(serialize = "GEODETIC", serialize = "Geodetic")]
    #[default]
    Geodetic,
    /// Earth fixed & low precision
    #[strum(serialize = "NON GEODETIC", serialize = "NonGeodetic")]
    NonGeodetic,
    /// Generated from network
    #[strum(serialize = "NON PHYSICAL", serialize = "NonPhysical")]
    NonPhysical,
    /// Orbiting space vehicle
    #[strum(serialize = "SPACE BORNE", serialize = "Spaceborne")]
    Spaceborne,
    /// Aircraft, balloon..
    #[strum(serialize = "AIR BORNE", serialize = "Airborne")]
    Airborne,
    /// Mobile water craft
    #[strum(serialize = "WATER CRAFT", serialize = "Watercraft")]
    Watercraft,
    /// Mobile terrestrial vehicle
    #[strum(serialize = "GROUND CRAFT", serialize = "Groundcraft")]
    Groundcraft,
    /// Fixed on water surface
    #[strum(serialize = "FIXED BUOY", serialize = "FixedBuoy")]
    FixedBuoy,
    /// Floating on water surface
    #[strum(serialize = "FLOATING BUOY", serialize = "FloatingBuoy")]
    FloatingBuoy,
    /// Floating on ice
    #[strum(serialize = "FLOATING ICE", serialize = "FloatingIce")]
    FloatingIce,
    /// Fixed on glacier
    #[strum(serialize = "GLACIER", serialize = "Glacier")]
    Glacier,
    /// Rockets, shells, etc..
    #[strum(serialize = "BALLISTIC", serialize = "Ballistic")]
    Ballistic,
    /// Animal carrying a receiver
    #[strum(serialize = "ANIMAL", serialize = "Animal")]
    Animal,
    /// Human being carrying a receiver
    #[strum(serialize = "HUMAN", serialize = "Human")]
    Human,
}

impl GeodeticMarker {
    /// Returns a GeodeticMarker with given "name".
    pub fn with_name(&self, name: &str) -> Self {
        let mut s = self.clone();
        s.name = name.to_string();
        s
    }
    /// Returns marker "number" in standardized format
    pub fn number(&self) -> Option<String> {
        let id = self.identification?;
        let monument = self.monument?;
        Some(format!("{:05}M{:03}", id, monument))
    }
    /// Returns a GeodeticMarker with "number" only if it matches standardized format.
    pub fn with_number(&self, content: &str) -> Self {
        let mut s = self.clone();
        if content.len() == 9 {
            if content.chars().nth(5) == Some('M') {
                if let Ok(id) = u32::from_str_radix(&content[..5], 10) {
                    if let Ok(monument) = u16::from_str_radix(&content[7..], 10) {
                        s.identification = Some(id);
                        s.monument = Some(monument);
                    }
                }
            }
        }
        s
    }
}

#[cfg(test)]
mod test {
    use super::GeodeticMarker;
    #[test]
    fn marker_number() {
        let marker = GeodeticMarker::default();
        let marker = marker.with_number("10118M001");
        assert_eq!(marker.number(), Some("10118M001".to_string()));
    }
}
