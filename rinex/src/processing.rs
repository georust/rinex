use super::Rinex;
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

/// DoubleDiffContext is a structure to help perform
/// RINEX double differentiation operation.
/// It holds Observation and Navigation data,
/// ideally sampled at the same time.
#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub struct DoubleDiffContext {
    pub observations: Rinex,
    pub ephemeris: Rinex,
}

impl DoubleDiffContext {
    /// Builds a DoubleDiffContext from given observation
    /// and navigation data files. Only phase data are kept from observed data.
    /// Only Ephemeris are kept among navigation data.
    pub fn from_file (observations: &str, ephemeris: &str) -> Result<Self, Error> {
        let observations = Rinex::from_file(observations)?;
        let ephemeris = Rinex::from_file(ephemeris)?;
        if observations.is_observation_rinex() {
            if ephemeris.is_navigation_rinex() {
                Ok(Self {
                    observations,
                    ephemeris,
                })
            } else {
                Err(Error::NotNavigationRinex)
            }
        } else {
            Err(Error::NotObservationRinex)
        }
    }
    /// Builds a new DoubleDiffContext.
    /// Only Ephemeris and Phase observations are kept, to simplify
    /// the following operation
    pub fn new (observations: Rinex, ephemeris: Rinex) -> Result<Self, Error> {
        if observations.is_observation_rinex() {
            if ephemeris.is_navigation_rinex() {
                Ok(Self {
                    observations,
                    ephemeris,
                })
            } else {
                Err(Error::NotNavigationRinex)
            }
        } else {
            Err(Error::NotObservationRinex)
        }
    }
    /// Returns a DoubleDiffContext with given Observation data
    /// where we only retain raw Phase Observations to simplify
    /// the following operation
    pub fn with_observations(&self, observations: Rinex) -> Result<Self, Error> {
        if observations.is_observation_rinex() {
            Ok(Self {
                observations,
                ephemeris: self.ephemeris.clone(),
            })
        } else {
            Err(Error::NotObservationRinex)
        }
    }
    /// Returns a DoubleDiffContext with given Navigation data
    /// where we only retain Ephemeris data
    pub fn with_ephemeris(&self, ephemeris: Rinex) -> Result<Self, Error> {
        if ephemeris.is_navigation_rinex() {
            Ok(Self {
                observations: self.observations.clone(),
                ephemeris,
            })
        } else {
            Err(Error::NotNavigationRinex)
        }
    }
    
    /// Computes the double difference between `self` and `rhs` Observation RINEX.
    /// The performed operation is `self.observations` - `rhs`.
    /// Only phase data is modified in `self.observations`, other observables are preserved.
    /// Only shared epochs and shared vehicules are preserved in `self`.
    /// Only double differenced phase data are left out in `self`, as far 
    /// as phase observables are concerned.
    /// This operation cancels out the local receiver clock effects.
    ///
    /// Example:
    /// ```
    /// use rinex::*;
    /// // this only serves as an API demonstration
    /// // ideally all files involved share common epochs and identical sample rate
    /// let mut rnx_a = 
    ///     Rinex::from_file("../test_resources/OBS/V2/aopr0010.17o")
    ///         .unwrap();
    /// let mut rnx_b = 
    ///     Rinex::from_file("../test_resources/OBS/V2/rovn0010.21o")
    ///         .unwrap();
    /// let nav =
    ///     Rinex::from_file("../test_resources/NAV/V2/amel0010.21g")
    ///         .unwrap();
    /// // compute the double difference, to cancel out ionospheric/atmospheric and local induced biases
    /// rnx_a
    ///     .double_diff_mut(&rnx_b, &nav);
    /// // Remember only phase data get modified, other observables are untouched
    /// ```
    pub fn double_diff_mut (&mut self, rhs: &Self) -> Result<(), Error> {
        if !rhs.is_observation_rinex() {
            return Err(Error::NotObservationRinex)
        }
        self.diff_mut(rhs)?; // compute single diff (in place)
        let record = self.record // grab `self` record
            .as_mut_obs()
            .unwrap();
        let mut zenith = self.ephemeris.clone()
            .space_vehicules_closest_to_zenith();
        // filter non common epochs out, 
        // to make reference retrieval easier
        zenith.retain(|e, _| {
            let mut epoch_is_shared = false;
            for (ee, _) in record.iter() {
                if ee == e {
                    epoch_is_shared = true;
                    break
                }
            }
            epoch_is_shared
        });
        // drop constellations in self where no reference vehicules could be determined,
        // due to missing related `nav` ephemeris
        record.retain(|e, (_, vehicules)| {
            let zenith_vehicules = zenith.get(e)
                .unwrap(); // already filtered out
            vehicules.retain(|sv, _| {
                let mut constell_found = false;
                for vehicule in zenith_vehicules {
                    if vehicule.constellation == sv.constellation {
                        constell_found = true;
                        break
                    }
                }
                constell_found
            });
            vehicules.len() > 0
        });
        // browse `self` 
        for (e, (_, svs)) in record.iter_mut() {
            // reference vehicules
            let ref_vehicules = zenith.get(e)
                .unwrap(); // already left out
            // this structure will hold reference phase data
            // accross vehicules. We store 1 reference data,
            // per valid Phase observable,
            // because there might be several phase observables in multi carrier contexts.
            let mut references: HashMap<sv::Sv, HashMap<String, f64>> = HashMap::new(); 
            // grab reference phase data
            for (sv, observables) in svs.iter() {
                if ref_vehicules.contains(sv) {//this vehicule is identified as ref.
                    let mut inner: HashMap<String, f64> = HashMap::new(); 
                    for (observable, data) in observables.iter() {
                        if is_phase_carrier_obs_code!(observable) {
                            // this is a phase observation
                            inner.insert(observable.to_string(), data.obs);
                        }
                    }
                    references.insert(sv.clone(), inner); 
                }
            }
            // drop the reference vehicule observed phase
            // so we're only left with data ready to be dual differenced
            // as far as phase is concerned
            svs.retain(|sv, observables| { 
                let mut preserve = true;
                for ref_sv in references.keys() {
                    if ref_sv == sv { // this vehicule is identified as ref.
                        observables.retain(|obscode, _| { // drop phase data
                            !is_phase_carrier_obs_code!(obscode)
                        });
                        preserve = observables.len() > 0
                    }
                }
                preserve
            });
            // compute double diff
            for (sv, observables) in svs.iter_mut() {
                // compute as long as we have 1 ref for related constellation
                for (ref_vehicule, ref_observables) in references.iter() {
                    if sv.constellation == ref_vehicule.constellation {
                        // compute as long as it's phase data
                        for (observable, data) in observables.iter_mut() {
                            if !is_phase_carrier_obs_code!(observable) {
                                // compute as long as we have a reference for this observable
                                for (ref_obs, ref_data) in ref_observables.iter() {
                                    if observable == ref_obs { // observable match
                                        data.obs -= ref_data // remove ref phase value
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Immutable implementation of [double_diff_mut].
    pub fn double_diff (&self, rhs: &Self, nav: &Self) -> Result<Self, DiffError> {
        let mut c = self.clone();
        c.double_diff_mut(rhs, nav)?;
        Ok(c)
    }
    
}
