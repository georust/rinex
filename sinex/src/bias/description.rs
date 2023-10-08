//use thiserror::Error;
//use strum_macros::EnumString;
use crate::bias;
use rinex::constellation::Constellation;
use std::collections::HashMap;
//use crate::datetime::{parse_datetime, ParseDateTimeError};

#[derive(Debug, Clone, Default)]
pub struct Description {
    /// Observation Sampling: sampling interval in seconds
    pub sampling: Option<u32>,
    /// Parameter Spacing: spacing interval in seconds,
    /// used for parameter representation
    pub spacing: Option<u32>,
    /// Method used to generate the bias results
    pub method: Option<bias::DeterminationMethod>,
    /// See [bias::header::BiasMode]
    pub bias_mode: bias::header::BiasMode,
    /// TimeSystem, see [bias::TimeSystem]
    pub system: bias::TimeSystem,
    /// Receiver clock reference GNSS
    pub rcvr_clock_ref: Option<Constellation>,
    /// Satellite clock reference observables:
    /// list of observable codes (standard 3 letter codes),
    /// for each GNSS in this file.
    /// Must be provided if associated bias results are consistent
    /// with the ionosphere free LC, otherwise, these might be missing
    pub sat_clock_ref: HashMap<Constellation, Vec<String>>,
}

impl Description {
    pub fn with_sampling(&self, sampling: u32) -> Self {
        Self {
            sampling: Some(sampling),
            spacing: self.spacing,
            method: self.method.clone(),
            bias_mode: self.bias_mode.clone(),
            system: self.system.clone(),
            rcvr_clock_ref: self.rcvr_clock_ref,
            sat_clock_ref: self.sat_clock_ref.clone(),
        }
    }
    pub fn with_spacing(&self, spacing: u32) -> Self {
        Self {
            sampling: self.sampling,
            spacing: Some(spacing),
            method: self.method.clone(),
            bias_mode: self.bias_mode.clone(),
            system: self.system.clone(),
            rcvr_clock_ref: self.rcvr_clock_ref,
            sat_clock_ref: self.sat_clock_ref.clone(),
        }
    }
    pub fn with_method(&self, method: bias::DeterminationMethod) -> Self {
        Self {
            sampling: self.sampling,
            spacing: self.spacing,
            method: Some(method),
            bias_mode: self.bias_mode.clone(),
            system: self.system.clone(),
            rcvr_clock_ref: self.rcvr_clock_ref,
            sat_clock_ref: self.sat_clock_ref.clone(),
        }
    }
    pub fn with_bias_mode(&self, mode: bias::header::BiasMode) -> Self {
        Self {
            sampling: self.sampling,
            spacing: self.spacing,
            method: self.method.clone(),
            bias_mode: mode,
            system: self.system.clone(),
            rcvr_clock_ref: self.rcvr_clock_ref,
            sat_clock_ref: self.sat_clock_ref.clone(),
        }
    }
    pub fn with_time_system(&self, system: bias::TimeSystem) -> Self {
        Self {
            sampling: self.sampling,
            spacing: self.spacing,
            method: self.method.clone(),
            bias_mode: self.bias_mode.clone(),
            system,
            rcvr_clock_ref: self.rcvr_clock_ref,
            sat_clock_ref: self.sat_clock_ref.clone(),
        }
    }
    pub fn with_rcvr_clock_ref(&self, clock_ref: Constellation) -> Self {
        Self {
            sampling: self.sampling,
            spacing: self.spacing,
            method: self.method.clone(),
            bias_mode: self.bias_mode.clone(),
            system: self.system.clone(),
            rcvr_clock_ref: Some(clock_ref),
            sat_clock_ref: self.sat_clock_ref.clone(),
        }
    }
    pub fn with_sat_clock_ref(&self, c: Constellation, observable: &str) -> Self {
        Self {
            sampling: self.sampling,
            spacing: self.spacing,
            method: self.method.clone(),
            bias_mode: self.bias_mode.clone(),
            system: self.system.clone(),
            rcvr_clock_ref: self.rcvr_clock_ref,
            sat_clock_ref: {
                let mut map = self.sat_clock_ref.clone();
                if let Some(codes) = map.get_mut(&c) {
                    if !codes.contains(&observable.to_string()) {
                        codes.push(observable.to_string());
                    }
                } else {
                    map.insert(c, vec![observable.to_string()]);
                }
                map
            },
        }
    }
}
