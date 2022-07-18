//use thiserror::Error;
//use strum_macros::EnumString;
//use std::collections::HashMap;
//use rinex::constellation::Constellation;

pub mod header;
pub mod description;

/*
/// Troposphere record content,
/// is a list of recording station coordinates,
/// and a list of Troposphere Solutions
#[derive(Debug, Clone)]
pub struct Record {
    /// Site Identification
    pub site_id: site::SiteIdentification,
    /// Site receiver info
    pub receiver: Option<site::Receiver>,
    /// Site antenna info
    pub antenna: Option<site::Antenna>,
    /// Site coordinates info
    pub site_coords: Option<site::Coordinates>,
    /// Site eccentricity info
    pub site_ecc: Option<site::Eccentricity>,
    /// Solutions
    pub solutions: Vec<Solution>,
}*/

/*
/// Coordinates gives the coordinates of a station
/// that provided one or several solutions in this file
#[derive(Debug, Clone)]
pub struct Coordinates {
    /// Site name (station name)
    pub site: String,
    /// Physical monument used at a site
    pub point_code: String,
    /// Number of solutions
    pub soln: u32,
    /// Technique used to generate the Troposphere
    /// solutions.
    pub technique: Technique,
    /// Station coordinates in [ddeg]
    pub coordinates: (f64,f64,f64),
    /// Coordinates system
    pub system: String,
    /// Optional remark
    pub remark: Option<String>,
}

/// Troposphere solution
#[derive(Debug, Clone)]
pub struct Data {
    /// Each value we encounter in the record
    value: f64,
    /// Optionnal stddev of the previous value
    stddev: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum Solution {
    /// ZTD: Tropospheric zenith total delay 
    ZTD(Data),
    /// ZWD: Tropospheric zenith wet delay 
    ZWD(Data),
    /// ZHD: Tropospheric zenith dry/hydrostatic delay
    ZHD(Data),
    /// Tropospheric total gradient - Northern component 
    /// (wet + dry parts)
    GNTotal(Data),
    /// TODO
    GNWet(Data),
    /// TODO
    GNDry(Data),
    /// TODO
    GETotal(Data),
    /// TODO
    GEWet(Data),
    /// TODO
    GEDry(Data),
    /// Integrated water vapour [kg.m^-2]
    IWV(Data),
}

#[derive(Error, Debug)]
pub enum ParseCoordinatesError {
    #[error("failed to parse sol#n")]
    ParseSOLnError(#[from] std::num::ParseIntError),
    #[error("failed to parse (x,y,z)")]
    ParseCoordError(#[from] std::num::ParseFloatError),
}

impl std::str::FromStr for Coordinates {
    type Err = ParseCoordinatesError; 
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        let (site, rem) = content.split_at(5);
        let (point_code, rem) = rem.split_at(3);
        let (soln, rem) = rem.split_at(5);
        let (t, rem) = rem.split_at(2);
        let (x, rem) = rem.split_at(13);
        let (y, rem) = rem.split_at(13);
        let (z, rem) = rem.split_at(13);
        let (system, rem) = rem.split_at(7);
        let remark = rem.clone();
        Ok(Coordinates {
            site: site.trim().to_string(),
            point_code: poit_code.trim().to_string(),
            technique: Technique::from_str(t.trim())?,
            soln: u32::from_str_radix(soln, 10)?,
            coordinates: (
                f64::from_str(x)?,  
                f64::from_str(y)?,  
                f64::from_str(z)?,  
            ),
            system: system.trim().to_string(),
            remark: remark.trim().to_string(),
        })
    }
}
*/
