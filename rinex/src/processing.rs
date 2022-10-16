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

/// Structure to perform the Double RINEX differentiation operation,
/// to cancel both atmospheric and local clock induced biases.
#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub struct DoubleDiffContext {
    /// Reference observations.
    /// Reference is B in RNX(A) - RNX(B)
    pub observations: Rinex,
    /// Reference phase data, determined from the provided
    /// Ephemeris context. One vehicule per constellation
    /// serves as reference point. We support N carrier frequencies.
    reference_phases: BTreeMap<Epoch, HashMap<Constellation, HashMap<String, f64>>>,
}

impl DoubleDiffContext {
    /// Builds a DoubleDiffContext from given observation
    /// and ephemeris data files. Ephemeris context is determined
    /// at this point.
    pub fn from_files (observations: &str, ephemeris: &str) -> Result<Self, Error> {
        let observations = Rinex::from_file(observations)?;
        let ephemeris = Rinex::from_file(ephemeris)?;
        let mut context = Self::with_reference_observations(&observations)?;
        context.set_ephemeris_context(&ephemeris)?;
        Ok(context)
    }
    /// Builds a new DoubleDiffContext.
    pub fn new (observations: &Rinex) -> Result<Self, Error> {
        Ok(Self::with_reference_observations(observations)?)
    }
    /// Returns Self with given reference Observations 
    pub fn with_reference_observations (observations: &Rinex) -> Result<Self, Error> {
        if observations.is_observation_rinex() {
            Ok(Self {
                observations: observations.clone(),
                // reference points not determined yet
                reference_phases: BTreeMap::new(), //with_capacity(record.len()),
            })
        } else {
            Err(Error::NotObservationRinex)
        }
    }
    /// Sets the Ephemeris context.
    /// In order to simplify following operations
    /// and post operation analysis,
    /// observations are severely reworked:
    ///  * sample rate is matched to ephemeris sample rate
    ///  * only Phase observations are preserved,
    ///  * reference phase data are extracted internally and dropped from record content
    pub fn set_ephemeris_context(&mut self, ephemeris: &Rinex) -> Result<(), Error> {
        if ephemeris.is_navigation_rinex() {
            // determine reference vehicules
            let reference_vehicules = ephemeris
                .space_vehicules_best_elevation_angle();
            // start from Observations context fresh copy
            self.reference_phases.clear();
            let record = self.observations.record
                .as_mut_obs()
                .unwrap();
            // rework sample rate 
            // extract reference phase data
            // remove reference phase data from record
            record.retain(|e, (_, vehicules)| {
                if let Some(ref_vehicules) = reference_vehicules.get(e) {
                    let mut inner: HashMap<Constellation, HashMap<String, f64>> = HashMap::new();
                    // drop reference vehicules from record
                    vehicules.retain(|sv, observations| {
                        if ref_vehicules.contains(sv) {
                            // grab reference data to be dropped
                            let mut phase_data: HashMap<String, f64> = HashMap::new();
                            for (observation, data) in observations {
                                if is_phase_carrier_obs_code!(observation) {
                                    phase_data.insert(observation.to_string(), data.obs);
                                }
                            }
                            if phase_data.len() > 0 {
                                // build reference phase data
                                inner.insert(sv.constellation, phase_data);
                            }
                            false
                        } else {
                            // only preserve phase data
                            observations.retain(|observation, _| {
                                is_phase_carrier_obs_code!(observation)
                            });
                            observations.len() > 0 
                        }
                    });
                    if vehicules.len() > 0 {
                        if inner.len() > 0 {
                            // build reference phase data
                            self.reference_phases.insert(*e,
                                inner);
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false // No reference vehicule for current epoch
                        // 2D diff would not be feasible
                       // ==> drop current epoch
                }
            });
            Ok(())
        } else {
            Err(Error::NotNavigationRinex)
        }
    }
 
    /// Computes the double difference `lhs` - `self`,
    /// RNX(a) - RNX(b), self is considered the reference.
    /// This operation cancels out ionospheric/atmospheric biases
    /// and the local receiver clock effects.
    /// This can only invoked once per Observation reference context,
    /// because internal data is modified in place:
    ///   * sample rate is adjusted to match `lhs` epochs
    ///   * only shared observations are preserved. If 
    ///   `lhs` is missing some carrier signals for instance,
    ///   observations get dropped in place
    /// If you want to perform as many 2D diff as you want,
    /// use [Self::double_diff] immutable implementation,
    /// which preserves the observation context,
    /// at the expense of a memcopy.
    ///
    /// Example:
/*
    /// ```
    /// use rinex::*;
    /// use rinex::processing::DoubleDiffContext;
    /// // In real use case, Ephemeris and Reference Observations have
    /// // a consistent sample rate. 
    /// // The DoubleDiffContext is smart enough to rework the sample rate
    /// // at the expence of losing Observations, if Ephemeris do not share
    /// // common epochs
    /// let mut context = DoubleDiffContext::from_files(
    ///    // provide reference observations
    ///    "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz",
    ///    // provide Ephemeris context
    ///    // so reference phase points can be determined
    ///    "../test_resources/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz")
    ///    .unwrap();
    /// // Perform 2D differentiation. Previously given Observations
    /// // are considered reference points (lhs - self).
    /// // Once again, ideally provide identical sampling context,
    /// // otherwise record is once again shrinked to fit
    /// let lhs = Rinex::from_file(
    ///     "../test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz")
    ///     .unwrap();
    /// context.double_diff_mut(&lhs)
    ///     .unwrap();
    /// // Post processing analysis
    /// let record = context.observations
    ///    .record
    ///    .as_obs()
    ///    .unwrap();
    /// for (e, (clk_offset, vehicules)) in record.iter() {
    ///    for (sv, observations) in vehicules {
    ///       // Vehicule used as reference is lost at this point
    ///       for (observable, data) in observations {
    ///          // Non phase observables are lost at this point
    ///          // We're only left with 2D differenced phase `data`
    ///       }
    ///    }
    /// }
    /// ```
*/
    pub fn double_diff_mut (&mut self, lhs: &Rinex) -> Result<(), Error> {
        if let Some(lhs_record) = lhs.record.as_obs() {
            // compute 1D diff
            // this will rework sampling rate to provided observations
            // and also only retains matching observables
            self.observations 
                .observation_diff_mut(lhs)?;
            // now proceed to 2D diff
            let record = self.observations
                .record
                .as_mut_obs()
                .unwrap();
            for (e, (_, vehicules)) in record.iter_mut() {
                let reference_vehicules = self.reference_phases.get(e)
                    .unwrap(); // already shrinked to fit
                for (vehicule, observations) in vehicules.iter_mut() {
                    // grab reference points for current constellation system
                    let reference_phase = reference_vehicules
                        .get(&vehicule.constellation)
                        .unwrap();
                    // substract reference point to current observation
                    for (observation, data) in observations.iter_mut() {
                        let reference_data = reference_phase.get(observation) 
                            .unwrap(); // already shrinked to fit
                        // substract reference phase point
                        data.obs -= reference_data;
                    }
                }
            }
            Ok(())
        } else {
            Err(Error::NotObservationRinex)
        }
    }

    /// Immutable 2D differentiation operation.
    /// This operation is less efficient compare to [Self::double_diff_mut],
    /// due to some extra memcopies,
    /// but allows as many invokations as you want, as long as the
    /// Ephemeris context remains the same.
    /// See [Self::double_diff_mut] for other examples.
    /// Example:
/*
    /// ```
    /// use rinex::*;
    /// use rinex::processing::DoubleDiffContext;
    /// let mut context = DoubleDiffContext::from_files(
    ///    // provide reference observations
    ///    "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz",
    ///    // provide Ephemeris context
    ///    // so reference phase points can be determined
    ///    "../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///    .unwrap();
    /// // Perform 2D differentiation. Previously given Observations
    /// // are considered reference points (lhs - self).
    /// let lhs = Rinex::from_file(
    ///     "../test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz")
    ///     .unwrap();
    /// let results = context.double_diff_mut(&lhs)
    ///     .unwrap(); // sane context
    /// // Post processing analysis
    /// for (e, (clk_offset, vehicules)) in results.iter() {
    ///    for (sv, observations) in vehicules.iter() {
    ///       // Vehicule used as reference is lost at this point
    ///       for (observable, data) in observations {
    ///          // Non phase observables are lost at this point
    ///          // We're only left with 2D differenced phase `data`
    ///       }
    ///    }
    /// }
    /// // Perform another 2D differentiation
    /// let lhs = Rinex::from_file(
    ///     "../test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz")
    ///     .unwrap();
    /// let results = context.double_diff_mut(&lhs)
    ///     .unwrap(); // sane context
    /// // [...]
    /// ```
*/
    pub fn double_diff (&self, lhs: &Rinex) -> Result<observation::Record, Error> {
        if let Some(lhs_record) = lhs.record.as_obs() {
            // compute 1D diff
            // this will rework sampling rate to provided observations
            // and also only retains matching observables
            let mut observations = self.observations 
                .observation_diff(lhs)?;
            // now proceed to 2D diff
            let mut record = observations
                .record
                .as_mut_obs()
                .unwrap();
            for (e, (_, vehicules)) in record.iter_mut() {
                let reference_vehicules = self.reference_phases.get(e)
                    .unwrap(); // already shrinked to fit
                for (vehicule, observations) in vehicules.iter_mut() {
                    // grab reference points for current constellation system
                    let reference_phase = reference_vehicules
                        .get(&vehicule.constellation)
                        .unwrap();
                    // substract reference point to current observation
                    for (observation, data) in observations.iter_mut() {
                        let reference_data = reference_phase.get(observation) 
                            .unwrap(); // already shrinked to fit
                        // substract reference phase point
                        data.obs -= reference_data;
                    }
                }
            }
            Ok(record.clone())
        } else {
            Err(Error::NotObservationRinex)
        }
    }
}
