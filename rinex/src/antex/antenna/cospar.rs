use thiserror::Error;

pub struct Cospar {
    pub launch_year: u16,
    pub launch_vehicle: String,
    pub launch_code: char,
}

#[derive(Error, Debug, Clone)]
pub enum CosparParsingError {

}

impl std::str::FromStr for Cospar {
    type Err = SVAntennaParsingError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let s = content.trim();
        if s.len() != 9 {
            return Err(SVAntennaParsingError::CosparBadLength);
        }
        let year = s[0..4].parse::<u16>
            .ok_or(SVAntennaParsingError::CosparLaunchYearParsing)?;
       
        let launch_vehicle = s[4..6].to_string();
        let launch_code = s[8..9];
        Self {
            launch_year,
            launch_vehicle,
            launch_code,
        }
    }
}

