use super::*;
use thiserror::Error;

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
            let mut observation = observation.clone();
            if navigation.is_navigation_rinex() {
                let mut navigation = navigation.clone();
                if let Some(obs_rec) = observation.record.as_mut_obs() {
                    if let Some(nav_rec) = navigation.record.as_mut_nav() {
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

                        // [OBS] rework sample rate
                        //       and retain shared vehicules only
                        obs_rec.retain(|e, (_, obs_vehicules)| {
                            if let Some(classes) = nav_rec.get(e) {
                                obs_vehicules.retain(|sv, _| {
                                    let mut found = false;
                                    for (_, frames) in classes {
                                        for fr in frames {
                                            let (_, nav_sv, _) = fr.as_eph()
                                                .unwrap();
                                            found |= nav_sv == sv;
                                        }
                                    }
                                    found
                                });
                                obs_vehicules.len() > 0
                            } else {
                                false // NAV does not share this epoch
                            }
                        });
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
    /// Results are MP_x where _x is the carrier signal, for instance MP_L1 for L1 carrier
    /// is computed by differentiating L1 against L2.
    /// Results are sorted by Elevation angle.
    /// Cf. p2 
    /// <https://www.taoglas.com/wp-content/uploads/pdf/Multipath-Analysis-Using-Code-Minus-Carrier-Technique-in-GNSS-Antennas-_WhitePaper_VP__Final-1.pdf>.
    /// Currently, we set K_i = n_i = 0 in the calculation.
    pub fn code_multipaths(&self) -> HashMap<String, HashMap<Sv, HashMap<i8, f64>>> {
        let mut result: HashMap<String, HashMap<Sv, HashMap<i8, f64>>> = HashMap::new();
        //let mut sv_map: HashMap<Sv, HashMap<String, f64>> = HashMap::new();
        //let mut code_map: HashMap<String, f64> = HashMap::new();
        //let mut data: HashMap<Channel, (String, f64, f64)> = HashMap::new(); // to store Phase/PR per observation
        if let Some(obs_record) = self.observation.record.as_obs() {
            if let Some(nav_record) = self.navigation.record.as_nav() {
                // pre calculate constant ratios/weight
                //   for each MPx x=carrier code
                let ratios = [
                    ("L1", 2.0* Channel::L2.carrier_frequency_mhz().powf(2.0) / (Channel::L1.carrier_frequency_mhz().powf(2.0) - Channel::L2.carrier_frequency_mhz().powf(2.0))),
                    ("L2", 2.0* Channel::L1.carrier_frequency_mhz().powf(2.0) / (Channel::L2.carrier_frequency_mhz().powf(2.0) - Channel::L1.carrier_frequency_mhz().powf(2.0))),
                    ("L5", 2.0* Channel::L1.carrier_frequency_mhz().powf(2.0) / (Channel::L5.carrier_frequency_mhz().powf(2.0) - Channel::L1.carrier_frequency_mhz().powf(2.0))),
                ];
                let ratios: HashMap<_, _> = ratios
                    .into_iter()
                    .collect();
                
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
                                        }
                                        // grab all Phase and PR Observations per code 
                                        let mut data: HashMap<&str, HashMap<&str, f64>> = HashMap::new();
                                        for (obscode, obsdata) in observations {
                                            if is_pseudo_range_obs_code!(obscode) | is_phase_carrier_obs_code!(obscode) {
                                                let code = &obscode[1..obscode.len()];
                                                if let Some(data) = data.get_mut(code) {
                                                    data.insert(code, obsdata.obs);
                                                } else {
                                                    let mut map: HashMap<&str, f64> = HashMap::new();
                                                    map.insert(obscode, obsdata.obs);
                                                    data.insert(code, map);
                                                }
                                            }
                                        }
                                        
                                        for (code, data)  in data {
                                            if code == "L1" { // L1 must be referenced to L2
                                                if let Some(ref_data) = data.get("L2") {
                                                    if let Some(ratio) = ratios.get(code) {
                                                    }
                                                }
                                            } else { // all others must be referenced to L1
                                                if let Some(ref_data) = data.get("L1") {
                                                    if let Some(ratio) = ratios.get(code) {
                                                    }
                                                }
                                            }
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
