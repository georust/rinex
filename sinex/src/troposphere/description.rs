use thiserror::Error;
use strum_macros::EnumString;
use std::collections::HashMap;
use rinex::constellation::Constellation;
use crate::{parse_datetime, ParseDateTimeError};

#[derive(Debug, Clone)]
pub struct Description {
    /// Sampling rate [s]
    pub data_rate: Option<u32>,
    /// Sampling rate for all trop estimates [s]
    pub trop_rate: Option<u32>,
    /// TODO
    pub elevation_cutoff_angle: u32,
    /// Tropospheric Hydrostatic and Wet Mapping
    /// functions used
    pub mapping_functions: Vec<String>,
    /// 
    pub solution_fields: Vec<String>,
}

impl Default for Description {
    fn default() -> Self {
        Self {
            sampling: None,
            sampling_tropo: None,
            elevation_cutoff_angle: 0,
            mapping_functions: Vec::new(),
            solution_fields: Vec::new(),
        }
    }
}

impl Description {
    pub fn with_sampling_interval (&self, interval: u32) -> Self {
        let mut s = self.clone();
        s.sampling = Some(interval);
        s
    }
    pub fn with_tropo_sampling (&self, interval: u32) -> Self {
        let mut s = self.clone();
        s.sampling_tropo = Some(interval);
        s
    }
    pub fn with_elevation_angle (&self, angle: u32) -> Self {
        let mut s = self.clone();
        s.elevation_cutoff_angle = angle;
        s
    }
    pub fn with_mapping_function (&self, func: &str) -> Self {
        let mut s = self.clone();
        s.mapping_functions.push(func.to_string());
        s
    }
    pub fn with_solution_field (&self, field: &str) -> Self {
        let mut s = self.clone();
        s.solution_fields.push(field.to_string());
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
}
