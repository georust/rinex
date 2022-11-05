use super::*;
use thiserror::Error;
use crate::observation::record::ObservationData;

#[derive(Debug, Error)]
pub enum Error {
    #[error("file is not an Observation RINEX")]
    NotObservationRinex,
    #[error("file is not a Navigation RINEX")]
    NotNavigationRinex,
    #[error("failed to parse RINEX data")]
    RinexError(#[from] super::Error),
}

/// RINEX Processing usually requires combining
/// Observation RINEX to Navigation RINEX.
/// This structure allows forming such a context easily,
/// and exposes RINEX processing methods
#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub struct Context {
    /// Observation RINEX
    pub observation: Rinex,
    /// Navigation RINEX
    pub navigation: Rinex,
}

impl Context {
    /// Builds a processing Context.
    pub fn new(observation: Rinex, navigation: Rinex) -> Result<Self, Error> {
        if observation.is_observation_rinex() {
            if navigation.is_navigation_rinex() {
                let mut observation = observation.clone();
                let mut navigation = navigation.clone();
                if let Some(obs_rec) = observation.record.as_mut_obs() {
                    if let Some(nav_rec) = navigation.record.as_mut_nav() {
                        println!("NAV LEN {}", nav_rec.len());
                        // [NAV] rework sample rate
                        //       retain ephemeris only with shared vehicules only
                        nav_rec.retain(|e, classes| {
                            if let Some((_, obs_vehicules)) = obs_rec.get(e) {
                                classes.retain(|class, frames| {
                                    if *class == navigation::FrameClass::Ephemeris {
                                        frames.retain(|fr| {
                                            let (_, sv, _) = fr.as_eph()
                                                .unwrap();
                                            obs_vehicules.get(sv).is_some()
                                        });
                                        frames.len() > 0
                                    } else {
                                        false // retain EPH only
                                    }
                        
                                });
                                classes.len() > 0
                            } else {
                                false // OBS does not share this epoch
                            }
                        });
                        println!("NAVV LEN {}", nav_rec.len());
                        
                        println!("OBS LEN {}", obs_rec.len());

                        // [OBS] rework sample rate
                        //       and retain shared vehicules only
                        obs_rec.retain(|e, (_, obs_vehicules)| {
                            if let Some(classes) = nav_rec.get(e) {
                                obs_vehicules.retain(|sv, _| {
                                    let mut found = false;
                                    for (_, frames) in classes { // already sorted out
                                        for fr in frames {
                                            let (_, nav_sv, _) = fr.as_eph()
                                                .unwrap(); // already sorted out
                                            found |= nav_sv == sv;
                                        }
                                    }
                                    println!("FOUND {}", found);
                                    found
                                });
                                obs_vehicules.len() > 0
                            } else {
                                false // NAV does not share this epoch
                            }
                        });
                        println!("OBSS LEN {}", obs_rec.len());
                    }
                }
                Ok(Self {
                    observation,
                    navigation,
                })
            } else {
                Err(Error::NotNavigationRinex)
            }
        } else {
            Err(Error::NotObservationRinex)
        }
    }
    /// Builds a processing Context from two local files
    pub fn from_files (observations: &str, navigation: &str) -> Result<Self, Error> {
        let navigation = Rinex::from_file(navigation)?;
        let observation = Rinex::from_file(observations)?;
        Self::new(observation, navigation)
    }

    /// Calculates Code MultiPath (MP) ratios by combining
    /// Pseudo Range observations and Phase observations sampled on different carriers.
    /// Resulting MPx ratios are sorted per code. For instance, "1C" means MP ratio for code 
    /// C for this vehicule.
    /// Cf. page 2 
    /// <https://www.taoglas.com/wp-content/uploads/pdf/Multipath-Analysis-Using-Code-Minus-Carrier-Technique-in-GNSS-Antennas-_WhitePaper_VP__Final-1.pdf>.
    /// Currently, we set K_i = n_i = 0 in the calculation.
    /// Example: 
    /// ```
    /// ```
    pub fn code_multipaths(&self) -> HashMap<String, HashMap<Sv, Vec<(i8, f64)>>> {
        let mut result: HashMap<String, HashMap<Sv, Vec<(i8, f64)>>> = HashMap::new();
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
        result
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
