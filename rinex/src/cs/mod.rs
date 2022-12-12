use crate::{
    *,
    prelude::*,
    observation::*,
    channel::Channel,
};
use std::collections::HashMap;

/*
/*
 * Selects an algorithm for each vehicle and each observation
 */
pub (crate)fn select_algorithm(&self, consistency: u8, sv: Sv, record: &Record) -> HashMap<Sv, HashMap<String, bool>> {
    let mut count: HashMap<String, (u32, u32)>; // (nb_epoch_seen, total_nb_epoch) per code 
    let total_epochs = record.len();
    for ((epoch, _), (_, svs)) in record {
        for (sv, observations) in svs {
            let mut remaining = observations.keys().collect();
            for (code, (seen, total)) in count {
                if let Some(_) = observations.get(code) {
                    // code is seen again
                    seen += 1;
                    remaining.remove(code);
                }
                total += 1;
            }
            for k in remaining {
                if count.get(k).is_none() {
                    // new code, never before seen
                    count.insert(k.to_string(), (1, 1));
                }
            }
        }
        
        /*
         * if all codes are already above threshold
         * => end this loop early
         * ==> makes decision process quickness related to designed consistency
         */
        let mut above = true;
        for (code, (seen, _)) in count {
            above &= (seen * 100 / total_epochs as u32) > consistency as u32;
        }
        if above {
            break;
        }
    }
    count.iter()
        .map(|(code, (seen, total))| {
            (*code, (seen * 100 / total) > consistency as u32)
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub enum CsAlgorithm {
    /// Geometry Free reombination based method.
    /// Is only feasible if carrier phase was estimated for at least
    /// 3 carrier signals.
    GfCombinationAdvanced,
    /// Geometry Free reombination based method.
    /// This method is possible on modern RINEX where carrier phase was estimated
    /// for at least two carrier signals.
    /// The lower the sampling rate, the better this method will work.
    /// This is the prefered method amongst all other
    GfCombinationSimple,
    /// Melbourne Wubbena recombination based method.
    /// This method is possible on modern RINEX where carrier phase was estimated
    /// for at least two carrier signals.
    /// This is the preferred method.
    MwCombination,
    /// Preferred Fallback method for old RINEX context
    /// or 1D contexts. 
    SingleFrequencyDoppler,
    /// Fallback method, always feasible even on old RINEX
    /// or single channel receivers.
    /// Highly subject to parameters fine tuning
    SingleFrequencyRecombination,
}

impl Default for CsAlgorithm {
    fn default() -> Self {
        Self::GfCombination
    }
}

impl Into<u8> for CsAlgorithm {
    fn into(&self) -> u8 {
        match self {
            Self::GfCombinationAdvanced => 255, 
            Self::GfCombinationSimple => 255, 
            Self::MwCombinationMultipath => 128,
            Self::MwCombination => 64,
            Self::SingleFrequencyDoppler => 8,
            Self::SingleFrequencyRecombination => 1,
        }
    }
}

//impl PartialOrd for CsAlgorithm {}

impl CsAlgorithm {
    /// Returns true if self is feasible, on given context 
    pub (crate)fn is_feasible(&self, context: &HashMap<String, ObservationData>) -> bool {
        match self {
            Self::SingleFrequencyRecombination => {
                for (code, _) in context {
                    let carrier_nb = &code[1..1]; 
                    if is_phase_carrier_obs_code!(code) {
                        // search for a PR code
                        for (others, _) in context {
                            let other_carrier = &code[1..1];
                            if others != code { // different code
                                if carrier_nb == other_carrier { // same carrier
                                    if is_pseudo_range_obs_code!(code) {
                                        return true;
                                    }
                                }
                            }
                        }
                    } else if is_pseudo_range_obs_code!(code) {
                        // search for a PH code
                        for (others, _) in context {
                            let other_carrier = &code[1..1];
                            if others != code { // different code
                                if carrier_nb == other_carrier { // same carrier
                                    if is_phase_carrier_obs_code!(code) {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Self::SingleFrequencyDoppler => {
                let single_freq_ok = CsAlgorithm::SingleFrequency.is_feasible(context);
                if !single_freq_ok {
                    return false;
                }
                // search for doppler obs
                for (code, _) in context {
                    if is_doppler_obs_code!(code) {
                        return true;
                    }
                }
            },
            Self::GfCombinationSimple => {
                // identify at least 2 Carrier Phase
                let mut nb_phase = 0;
                for (code, _) in context {
                    if is_phase_carrier_obs_code!(code) {
                        nb_phase += 1;
                    }
                }
            },
            Self::GfCombinationAdvanced => {
                let simple_ok = CsAlgorithm::GfCombinationSimple.is_feasible(context);
                if !simple_ok {
                    return false;
                }
                // identify at least 3 different PH observations
                let mut phase_codes = 0;
                for (code, _) in context {
                    if is_phase_carrier_obs_code!(code) {
                        phase_codes += 1;
                    }
                }
                return phase_codes > 2;
            },
            Self::MwCombination => {

            }
            _ => false,
        }
        false
    }
}
*/

/*
#[derive(Debug, Clone)]
pub enum CsMethod {
    /// The calculator decides which method to use,
    /// by order of preference and decides to use this method
    /// for all vehicles in the provided context
    Auto,
    /// The calculator elects 1 method that suites best
    /// every single vehicle context
    AutoSv,
    /// The calculator uses given algorithm for every single Sv
    Prefered(CsAlgorithm),
    /// The calculator uses given algorithm if 1 method
    /// has not been determined as better suited.
    PreferedSv(CsAlgorithm),
}*/

#[derive(Debug, Clone)]
pub struct OptsThreshold {
    pub a0: f64,
    pub a1: f64,
}

impl Default for OptsThreshold {
    /// Builds defaults OptsThreshold, best suited for L1/L2 studies
    /// with ~1min observation interval
    fn default() -> Self {
        let a0 = 3.0 * (Channel::L2.carrier_wavelength() - Channel::L1.carrier_wavelength()) / 2.0;
        Self {
            a0, 
            a1: a0 / 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CsOpts {
    // /// Decision method
    // pub method: CsMethod,
    // /// Percentage of epoch of the total record
    // /// the method should remain feasible for it to be elected.
    // /// A lower percentage gives quicker election process.
    // pub method_consistency_pct: u8,
    /// Sample rate dependent threshold,
    /// in the form a0 - a1 e^-dt
    /// to control the detector sensitivity against symbol rate
    pub threshold: OptsThreshold,
}

impl Default for CsOpts {
    fn default() -> Self {
        Self {
            /*method: CsMethod::Auto,
            method_consistency_pct: 33,*/
            threshold: OptsThreshold::default(),
        }
    }
}

impl CsOpts {
    /*pub fn with_method(&self, method: CsMethod) -> Self {
        let mut s = self.clone();
        s.method = method;
        s
    }*/
    pub fn with_threshold(&self, threshold: OptsThreshold) -> Self {
        let mut s = self.clone();
        s.threshold = threshold;
        s
    }
    /// Returns detector sensitivity at current context, in meter, 
    /// based on given (a0, a1) parameters 
    pub fn sensitivity(&self, dt: Duration, sample_rate: Duration) -> f64 {
        self.threshold.a0 - self.threshold.a1 * (-dt.to_unit(hifitime::Unit::Second) / sample_rate.to_unit(hifitime::Unit::Second)).exp()
    }
    /// Returns detector sensitivity [in ideal data scenario], in meter, 
    /// based on given (a0, a1) parameters
    pub fn best_sensitivity(&self) -> f64 {
        self.threshold.a0 - self.threshold.a1 * (-1.0_f64).exp()
    }
}
