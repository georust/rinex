use super::*;
use navigation::*;
use thiserror::Error;
//use crate::observation::record::ObservationData;

#[derive(Debug, Error)]
pub enum Error {
    #[error("`base` data must be Observation RINEX for this operation")]
    NotObservationBase,
    #[error("`base` data must be Navigation RINEX for this operation")]
    NotNavigationBase,
    #[error("`rover` must be Observation RINEX for this operation")]
    NotObservationRover,
    #[error("`rover` must be Navigation RINEX for this operation")]
    NotNavigationRover,
    #[error("failed to parse RINEX data")]
    RinexError(#[from] super::Error),
}

/// Advanced RINEX processing algorithms require
/// combining two RINEX files together.
/// This structure helps forming such a context.
/// In the following, `base` is the reference RINEX,
/// and `rover` is the data to compare to "base".
/// Meaningn, when substracting A-B, B is always the "base" and A is the "rover".
/// To this day, only Observation/Observation or
/// Observation/Navigation associations are known and truly allowed.
#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub struct DiffContext {
    /// "base": reference RINEX
    pub base: Rinex,
    /// "rover": other RINEX
    pub rover: Rinex,
}

impl DiffContext {
    /// Builds a new DiffContext from two mutable RINEX.
    /// Sample rate is reworked to perfectly match:
    /// non shared epochs are dropped, to facilitate further processing.
    /// For Observation context: we only retain Phase and Pseudo Range observations,
    /// as we don't know of Differential analysis involving other observations.
    /// For Navigation context: we only retain Ephemeris frames,
    /// as we don't know of Differential analysis involving other frames.
    pub fn new(base: &Rinex, rover: &Rinex) -> Self {
        let mut base = base.clone();
        let mut rover = rover.clone();
        // match /adjust sample rates
        base.decim_match_mut(&rover);
        rover.decim_match_mut(&base);
        // For Navigation RINEX
        //  retain ephemeris frames only
        base.retain_navigation_ephemeris_mut();
        rover.retain_navigation_ephemeris_mut();
        // For Observation RINEX
        //  retain Phase and Pseudo Range observations only
        //    and only shared vehicules
        if let Some(record) = base.record.as_mut_obs() {
            record.retain(|e, (_, vehicules)| {
                vehicules.retain(|sv, observations| {
                    if let Some(obs_rov) = rover.record.as_obs() {
                        let (_, rov_vehicules) = obs_rov.get(e)
                            .unwrap();
                        let mut shared = false;
                        for (rov_sv, _) in rov_vehicules {
                            shared |= rov_sv == sv;
                        }
                        if shared {
                            observations.retain(|code, _| {
                                is_pseudo_range_obs_code!(code) || is_phase_carrier_obs_code!(code)
                            });
                            observations.len() > 0
                        } else {
                            false
                        }
                    } else if let Some(nav_rov) = rover.record.as_nav() {
                        let mut shared = false;
                        let rov_classes = nav_rov.get(e)
                            .unwrap();
                        for (rov_class, rov_frames) in rov_classes {
                            for rov_frame in rov_frames {
                                let (_, rov_sv, _) = rov_frame.as_eph()
                                    .unwrap();
                                shared |= rov_sv == sv;
                            }
                        }
                        if shared {
                            observations.retain(|code, _| {
                                is_pseudo_range_obs_code!(code) || is_phase_carrier_obs_code!(code)
                            });
                            observations.len() > 0
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                });
                vehicules.len() > 0 
            });
        }
        // For Navigation (ephemeris..) RINEX
        //  retain only shared vehicules
        else if let Some(record) = base.record.as_mut_nav() {
            record.retain(|e, classes| {
                classes.retain(|class, frames| {
                    frames.retain(|fr| {
                        let mut shared = false;
                        let (_, sv, _) = fr.as_eph()
                            .unwrap();
                        if let Some(obs_rec) = rover.record.as_obs() {
                            let (_, obs_vehicules) = obs_rec.get(e)
                                .unwrap();
                            for (obs_sv, _) in obs_vehicules {
                                shared |= obs_sv == sv;
                            }
                        } else if let Some(nav_rec) = rover.record.as_nav() {
                            let classes = nav_rec.get(e)
                                .unwrap();
                            for (class, frames) in classes {
                                for fr in frames {
                                    let (_, nav_sv, _) = fr.as_eph()
                                        .unwrap();
                                    shared |= nav_sv == sv;
                                }
                            }
                        }
                        shared
                    });
                    frames.len() > 0
                });
                classes.len() > 0
            });
        }
        /*
         * at this point "base" is ready for processing,
         * let's strip "rover" to identical vehicules to
         * to speed up and eventually facilitate further operations
         */
        if let Some(record) = rover.record.as_mut_obs() {
            record.retain(|e, (_, vehicules)| {
                vehicules.retain(|sv, _| {
                    let mut shared = false;
                    if let Some(obs_base) = base.record.as_obs() {
                        let (_, base_vehicules) = obs_base.get(e)
                            .unwrap();
                        for (base_sv, _) in base_vehicules {
                            shared |= base_sv == sv;
                        }
                    } else if let Some(nav_base) = base.record.as_nav() {
                        let (base_classes) = nav_base.get(e)
                            .unwrap();
                        for (class, frames) in base_classes {
                            for fr in frames {
                                let (_, nav_sv, _) = fr.as_eph()
                                    .unwrap();
                                shared |= nav_sv == sv;
                            }
                        }
                    }
                    false
                });
                vehicules.len() > 0
            });
        } else if let Some(record) = rover.record.as_mut_nav() {
            record.retain(|e, classes| {
                classes.retain(|class, frames| {
                    frames.retain(|fr| {
                        let mut shared = false;
                        let (_, sv, _) = fr.as_eph()
                            .unwrap();
                        if let Some(obs_base) = base.record.as_obs() {
                            let (_, base_vehicules) = obs_base.get(e)
                                .unwrap();
                            for (base_sv, _) in base_vehicules {
                                shared |= sv == base_sv;
                            }
                        } else if let Some(nav_base) = base.record.as_nav() {
                            let nav_classes = nav_base.get(e)
                                .unwrap();
                            for (class, frames) in nav_classes {
                                for fr in frames {
                                    let (_, base_sv, _) = fr.as_eph()
                                        .unwrap();
                                    shared |= sv == base_sv;
                                }
                            }
                        }
                        shared
                    });
                    frames.len() > 0
                });
                classes.len() > 0
            });
        }
        Self {
            base,
            rover,
        }
    }

    /// Builds Self from two local files
    pub fn from_files(fp: &str, rover_fp: &str) -> Result<Self, Error> {
        let rnx = Rinex::from_file(fp)?;
        let rover = Rinex::from_file(rover_fp)?;
        Ok(Self::new(&rnx, &rover))
    }

    /// Returns geometric biases (delta rho) in Eq(2) page 2
    /// which is are dominant biases in the cycle slip detection
    /// algorithm. This can only be performed
    /// against different carrier frequencies.
    /// Single difference returns Phase  
    /// substracting Phase observations between
    /// identical carrier signal. 
    pub fn geometric_biases(&self) -> Result<(), Error> {
        if !self.base.is_observation_rinex() {
            return Err(Error::NotObservationBase);
        }
        if !self.rover.is_observation_rinex() {
            return Err(Error::NotObservationRover);
        }
        let rec = self.rover.record.as_mut_obs()
            .unwrap();
        let rov_rec = self.base.record.as_obs()
            .unwrap();
        for (epoch, (_, vehicules)) in rec.iter_mut() {
            let (_, rov_vehicules) = rov_rec.get(&epoch)
                .unwrap();
            for (sv, vehicules) in vehicules.iter_mut() {
                let rov_observations = rov_vehicules.get(&sv)
                    .unwrap();
                for observation in observations {
                    if is_phase_carrier_obs_code!(
                    /*if is_phase_carrier_obs_code!(observation) {
                        // locate same observation in Rover data
                    }
                    if is_pseudo_range_obs_code!(observation) {

                    }*/
                }
            }
        }
        Ok(())
    }

    /// Calculates Code MultiPath (MP) ratios by combining
    /// Pseudo Range observations and Phase observations sampled on different carriers.
    /// `rnx` must be Observation RINEX for this operation.
    /// `rover` must be Navigation RINEX for this operation.
    ///
    /// Resulting MPx ratios are sorted per code. For instance, "1C" means MP ratio for code 
    /// C for this vehicule.
    /// Cf. page 2 
    /// <https://www.taoglas.com/wp-content/uploads/pdf/Multipath-Analysis-Using-Code-Minus-Carrier-Technique-in-GNSS-Antennas-_WhitePaper_VP__Final-1.pdf>.
    /// Currently, we set K_i = n_i = 0 in the calculation.
    pub fn code_multipaths(&self) -> Result<HashMap<String, HashMap<Sv, Vec<(i8, f64)>>>, Error> {
        if !self.base.is_navigation_rinex() {
            return Err(Error::NotNavigationBase);
        }
        if !self.rover.is_observation_rinex() {
            return Err(Error::NotObservationRover);
        }
            
        let result: HashMap<String, HashMap<Sv, Vec<(i8, f64)>>> = HashMap::new();
/*
        //TODO lazy_static please
        let known_codes = vec![
            "1A","1B","1C","1D","1W","1X","1Z","1P","1S","1L","1M",
            "2C","2W","2D","2S","2L","2P","2M",
            "3I","3X","3Q",
            "4A","4B","4X",
            "5A","5B","5C","5P","5I","5Q","5X",
            "6A","6B","6C","6Q","6X","6Z",
            "7D","7I","7P","7Q","7X",
            "8D","8P","8I","8Q","8X",
            "9A", "9B","9C","9X",
        ];
        
        if let Some(obs_record) = self.observation.record.as_obs() {
            if let Some(nav_record) = self.navigation.record.as_nav() {
                for (epoch, classes) in nav_record {
                    if let Some((_, obs_vehicules)) = obs_record.get(epoch) {
                        for (class, frames) in classes {
                            if *class == navigation::FrameClass::Ephemeris {
                                for frame in frames {
                                    let (_, sv, eph) = frame.as_eph()
                                        .unwrap(); // already sorted out
                                    if let Some(observations) = obs_vehicules.get(sv) {
                                        if let Some(elevation) = eph.elevation_angle() {
                                            let elevation = elevation.round() as i8;
                                            for code in &known_codes {
                                                // for each known code,
                                                //  we must have a C and L observation
                                                //   and also an L observation for 
                                                //     Carrier 2 if we're dealing with a 1x Code
                                                //     Carrier 1 for all others
                                                let c_code = "C".to_owned() + code;
                                                let l_code = "L".to_owned() + code;
                                                println!("C CODE \"{}\" L CODE \"{}\"", c_code, l_code); // DEBUG
                                                if let Some(c_data) = observations.get(&c_code) {
                                                    if let Some(l_data) = observations.get(&l_code) {
                                                        let ref_data: Vec<ObservationData> = 
                                                            observations.iter()
                                                            .filter_map(|(codes, _)| {
                                                                if c_code.contains("1") { // we're dealing with Carrer 1
                                                                    // we refer to Carrier 2 L code
                                                                    // ==> try to locate one
                                                                    for code in &known_codes {
                                                                        if code.contains("2") {
                                                                            let to_find = "L".to_owned() + code.clone();
                                                                            println!("L2 CODE to find \"{}\"", to_find); // DEBUG
                                                                        }
                                                                    }
                                                                } else {
                                                                    // when dealing with other carriers
                                                                    // we refer to Carrier 1  L code
                                                                    // ==> try to locate one
                                                                    for code in &known_codes {
                                                                        if code.contains("1") {
                                                                            let to_find = "L".to_owned() + code.clone();
                                                                            println!("L1 CODE to find \"{}\"", to_find); // DEBUG
                                                                        }
                                                                    }
                                                                }
                                                                None
                                                            })
                                                            .collect();
                                                    }
                                                }
                                            }
                                        } else {
                                            println!("NO ELEVATION!!"); // DEBUG
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
*/
        Ok(result)
    }
/*
        // preprocess ratios
        for ((nav_epoch, nav_frames), (obs_epoch, (_, observations))) in  

        for (epoch, vehicules) in record.iter() {
            sv_map.clear();
            for (sv, observations) in vehicules.iter() {
                code_map.clear();
                data.clear();
                for (observation, data) in observations {
                    // identify carrier channel of this observation
                    if let Ok(channel) = Channel::from_observable(sv.constellation, observation) {
                        if is_pseudo_range_obs_code!(observation) {
                            if let Some((_, pr)) = data.get_mut(channel) {
                                pr = observation.data; // insert PR
                            } else {
                                data.insert(channel, (0.0, observation.data)); // insert PR
                            }
                        } else if is_phase_carrier_obs_code!(observation) {
                            if let Some((ph, _)) = data.get_mut(channel) {
                                ph = observation.data; // insert PR
                            } else {
                                data.insert(channel, (observation.data, 0.0)); // insert PR
                            }
                        }
                    }
                }
                if let Some((pr1, ph1)) = data.get(Channel::L1) {
                    if let Some((pr2, ph2)) = data.get(Channel::L1) {
                    }
                }
            }
            if map.len() > 0 {
                result.insert(*epoch, map);
            }
        }
        result
    }
*/
}
