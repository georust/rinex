//! header line #1 parsing helper

use crate::ParsingError;
use crate::{DataType, OrbitType, Version};

pub(crate) fn is_header_line1(content: &str) -> bool {
    content.starts_with('#')
}

pub(crate) struct Line1 {
    pub version: Version,
    pub data_type: DataType,
    pub coord_system: String,
    pub orbit_type: OrbitType,
    pub agency: String,
}

impl std::str::FromStr for Line1 {
    type Err = ParsingError;
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if line.len() != 60 {
            return Err(ParsingError::MalformedH1);
        }
        Ok(Self {
            version: Version::from_str(&line[1..2])?,
            data_type: DataType::from_str(&line[2..3])?,
            coord_system: line[45..51].trim().to_string(),
            orbit_type: OrbitType::from_str(line[51..55].trim())?,
            agency: line[55..].trim().to_string(),
        })
    }
}

impl Line1 {
    pub(crate) fn to_parts(&self) -> (Version, DataType, String, OrbitType, String) {
        (
            self.version,
            self.data_type,
            self.coord_system.clone(),
            self.orbit_type,
            self.agency.clone(),
        )
    }
}
