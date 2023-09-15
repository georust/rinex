//! Velocity entry parsing
use crate::ParsingError;
use crate::{Epoch, Sv, Vector3D};
use std::collections::BTreeMap;

/*
 * SV velocities prediction estimates
 */
pub type VelocityRecord = BTreeMap<Epoch, BTreeMap<Sv, Vector3D>>;
/*
 * Clock rate of change record content
 */
pub type ClockRateRecord = BTreeMap<Epoch, BTreeMap<Sv, f64>>;

pub(crate) fn velocity_entry(content: &str) -> bool {
    content.starts_with('V')
}

pub(crate) struct VelocityEntry {
    sv: Sv,
    velocity: (f64, f64, f64),
    clock: Option<f64>,
}

impl std::str::FromStr for VelocityEntry {
    type Err = ParsingError;
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let mut clock: Option<f64> = None;
        let sv =
            Sv::from_str(line[1..4].trim()).or(Err(ParsingError::Sv(line[1..4].to_string())))?;
        let x = f64::from_str(line[4..18].trim())
            .or(Err(ParsingError::Coordinates(line[4..18].to_string())))?;
        let y = f64::from_str(line[18..32].trim())
            .or(Err(ParsingError::Coordinates(line[18..32].to_string())))?;
        let z = f64::from_str(line[32..46].trim())
            .or(Err(ParsingError::Coordinates(line[32..46].to_string())))?;

        if !line[45..52].trim().eq("999999.") {
            /*
             * Clock data present
             */
            let clk_data = f64::from_str(line[46..60].trim())
                .or(Err(ParsingError::Clock(line[46..60].to_string())))?;
            clock = Some(clk_data);
        }
        Ok(Self {
            sv,
            velocity: (x, y, z),
            clock,
        })
    }
}

impl VelocityEntry {
    pub fn to_parts(&self) -> (Sv, (f64, f64, f64), Option<f64>) {
        (self.sv, self.velocity, self.clock)
    }
}
