//! GNSS processing context definition.
use thiserror::Error;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use rinex::{
    carrier::Carrier,
    merge::{Error as RinexMergeError, Merge as RinexMerge},
    prelude::{Almanac, Epoch, GroundPosition, Observable, Rinex, TimeScale, SV},
    types::Type as RinexType,
    Error as RinexError,
};

use anise::{
    almanac::planetary::PlanetaryDataError, constants::frames::IAU_EARTH_FRAME,
    errors::AlmanacError, prelude::Frame,
};

#[cfg(feature = "sp3")]
use sp3::prelude::SP3;

use qc_traits::{
    processing::{Filter, Preprocessing, Repair, RepairTrait},
    Merge, MergeError,
};

/// Context Error
#[derive(Debug, Error)]
pub enum Error {
    #[error("almanac error")]
    Alamanac(#[from] AlmanacError),
    #[error("planetary data error")]
    PlanetaryData(#[from] PlanetaryDataError),
    #[error("failed to extend gnss context")]
    ContextExtensionError(#[from] MergeError),
    #[error("non supported file format")]
    NonSupportedFileFormat,
    #[error("failed to determine filename")]
    FileNameDetermination,
    #[error("invalid rinex format")]
    RinexError(#[from] RinexError),
    #[error("failed to extend rinex context")]
    RinexMergeError(#[from] RinexMergeError),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProductType {
    /// GNSS carrier signal observation in the form
    /// of Observation RINEX data.
    Observation,
    /// Meteo sensors data wrapped as Meteo RINEX files.
    MeteoObservation,
    /// DORIS measurements wrapped as special RINEX observation file.
    DORIS,
    /// Broadcast Navigation message as contained in
    /// Navigation RINEX files.
    BroadcastNavigation,
    /// High precision orbital attitudes wrapped in Clock RINEX files.
    HighPrecisionClock,
    /// Antenna calibration information wrapped in ANTEX special RINEX files.
    ANTEX,
    /// Precise Ionosphere state wrapped in IONEX special RINEX files.
    IONEX,
    #[cfg(feature = "sp3")]
    #[cfg_attr(docrs, doc(cfg(feature = "sp3")))]
    /// High precision clock data wrapped in SP3 files.
    HighPrecisionOrbit,
}

impl std::fmt::Display for ProductType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ANTEX => write!(f, "ANTEX"),
            Self::IONEX => write!(f, "IONEX"),
            Self::DORIS => write!(f, "DORIS RINEX"),
            Self::Observation => write!(f, "Observation"),
            Self::MeteoObservation => write!(f, "Meteo"),
            Self::HighPrecisionClock => write!(f, "High Precision Clock"),
            Self::BroadcastNavigation => write!(f, "Broadcast Navigation (BRDC)"),
            #[cfg(feature = "sp3")]
            Self::HighPrecisionOrbit => write!(f, "High Precision Orbit (SP3)"),
        }
    }
}

impl From<RinexType> for ProductType {
    fn from(rt: RinexType) -> Self {
        match rt {
            RinexType::ObservationData => Self::Observation,
            RinexType::NavigationData => Self::BroadcastNavigation,
            RinexType::MeteoData => Self::MeteoObservation,
            RinexType::ClockData => Self::HighPrecisionClock,
            RinexType::IonosphereMaps => Self::IONEX,
            RinexType::AntennaData => Self::ANTEX,
            RinexType::DORIS => Self::DORIS,
        }
    }
}

enum BlobData {
    /// RINEX content
    Rinex(Rinex),
    #[cfg(feature = "sp3")]
    /// SP3 content
    Sp3(SP3),
}

impl BlobData {
    /// Returns reference to inner RINEX data.
    pub fn as_rinex(&self) -> Option<&Rinex> {
        match self {
            Self::Rinex(r) => Some(r),
            _ => None,
        }
    }
    /// Returns mutable reference to inner RINEX data.
    pub fn as_mut_rinex(&mut self) -> Option<&mut Rinex> {
        match self {
            Self::Rinex(r) => Some(r),
            _ => None,
        }
    }
    /// Returns reference to inner SP3 data.
    #[cfg(feature = "sp3")]
    #[cfg_attr(docrs, doc(cfg(feature = "sp3")))]
    pub fn as_sp3(&self) -> Option<&SP3> {
        match self {
            Self::Sp3(s) => Some(s),
            _ => None,
        }
    }
    /// Returns mutable reference to inner SP3 data.
    #[cfg(feature = "sp3")]
    #[cfg_attr(docrs, doc(cfg(feature = "sp3")))]
    pub fn as_mut_sp3(&mut self) -> Option<&mut SP3> {
        match self {
            Self::Sp3(s) => Some(s),
            _ => None,
        }
    }
}

/// [QcContext] is a general structure capable to store most common
/// GNSS data. It is dedicated to post processing workflows,
/// precise timing or atmosphere analysis.
pub struct QcContext {
    /// Files merged into self
    files: HashMap<ProductType, Vec<PathBuf>>,
    /// Context blob created by merging each members of each category
    blob: HashMap<ProductType, BlobData>,
    /// Latest Almanac
    pub almanac: Almanac,
    /// Earth IAU/ECEF frame
    pub earth_iau_ecef: Frame,
}

impl QcContext {
    /// Initilize QcContext
    pub fn new() -> Result<Self, Error> {
        let almanac = Almanac::until_2035()?;
        let earth_iau_ecef = almanac.frame_from_uid(IAU_EARTH_FRAME)?;
        Ok(Self {
            almanac,
            earth_iau_ecef,
            blob: HashMap::new(),
            files: HashMap::new(),
        })
    }
    /// Build new [QcContext] with given [Almanac].
    pub fn new_almanac(almanac: Almanac) -> Result<Self, Error> {
        let earth_iau_ecef = almanac.frame_from_uid(IAU_EARTH_FRAME)?;
        Ok(Self {
            almanac,
            earth_iau_ecef,
            blob: HashMap::new(),
            files: HashMap::new(),
        })
    }
    /// Returns main [TimeScale] for Self
    pub fn timescale(&self) -> Option<TimeScale> {
        #[cfg(feature = "sp3")]
        if let Some(sp3) = self.sp3() {
            return Some(sp3.time_scale);
        }

        if let Some(obs) = self.observation() {
            let first = obs.first_epoch()?;
            Some(first.time_scale)
        } else if let Some(dor) = self.doris() {
            let first = dor.first_epoch()?;
            Some(first.time_scale)
        } else if let Some(clk) = self.clock() {
            let first = clk.first_epoch()?;
            Some(first.time_scale)
        } else if self.meteo().is_some() {
            Some(TimeScale::UTC)
        } else if self.ionex().is_some() {
            Some(TimeScale::UTC)
        } else {
            None
        }
    }
    /// Returns path to File considered as Primary product in this Context.
    /// When a unique file had been loaded, it is obviously considered Primary.
    pub fn primary_path(&self) -> Option<&PathBuf> {
        /*
         * Order is important: determines what format are prioritized
         * in the "primary" determination
         */
        for product in [
            ProductType::Observation,
            ProductType::DORIS,
            ProductType::BroadcastNavigation,
            ProductType::MeteoObservation,
            ProductType::IONEX,
            ProductType::ANTEX,
            ProductType::HighPrecisionClock,
            #[cfg(feature = "sp3")]
            ProductType::HighPrecisionOrbit,
        ] {
            if let Some(paths) = self.files(product) {
                /*
                 * Returns Fist file loaded in this category
                 */
                return paths.first();
            }
        }
        None
    }
    /// Returns name of this context.
    /// Context is named after the file considered as Primary, see [Self::primary_path].
    /// If no files were previously loaded, simply returns "Undefined".
    pub fn name(&self) -> String {
        if let Some(path) = self.primary_path() {
            path.file_name()
                .unwrap_or(OsStr::new("Undefined"))
                .to_string_lossy()
                // removes possible .crx ; .gz extensions
                .split('.')
                .next()
                .unwrap_or("Undefined")
                .to_string()
        } else {
            "Undefined".to_string()
        }
    }
    /// Returns reference to files loaded in given category
    pub fn files(&self, product: ProductType) -> Option<&Vec<PathBuf>> {
        self.files
            .iter()
            .filter_map(|(prod_type, paths)| {
                if *prod_type == product {
                    Some(paths)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }
    /// Returns mutable reference to files loaded in given category
    pub fn files_mut(&mut self, product: ProductType) -> Option<&Vec<PathBuf>> {
        self.files
            .iter()
            .filter_map(|(prod_type, paths)| {
                if *prod_type == product {
                    Some(paths)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }
    /// Returns reference to inner data of given category
    fn data(&self, product: ProductType) -> Option<&BlobData> {
        self.blob
            .iter()
            .filter_map(|(prod_type, data)| {
                if *prod_type == product {
                    Some(data)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }
    /// Returns mutable reference to inner data of given category
    fn data_mut(&mut self, product: ProductType) -> Option<&mut BlobData> {
        self.blob
            .iter_mut()
            .filter_map(|(prod_type, data)| {
                if *prod_type == product {
                    Some(data)
                } else {
                    None
                }
            })
            .reduce(move |k, _| k)
    }
    /// Returns reference to inner RINEX data of given category
    pub fn rinex(&self, product: ProductType) -> Option<&Rinex> {
        self.data(product)?.as_rinex()
    }
    /// Returns mutable reference to inner RINEX data of given category
    pub fn rinex_mut(&mut self, product: ProductType) -> Option<&mut Rinex> {
        self.data_mut(product)?.as_mut_rinex()
    }
    /// Returns reference to inner SP3 data
    #[cfg(feature = "sp3")]
    #[cfg_attr(docrs, doc(cfg(feature = "sp3")))]
    pub fn sp3(&self) -> Option<&SP3> {
        self.data(ProductType::HighPrecisionOrbit)?.as_sp3()
    }
    /// Returns reference to inner [ProductType::Observation] data
    pub fn observation(&self) -> Option<&Rinex> {
        self.data(ProductType::Observation)?.as_rinex()
    }
    /// Returns reference to inner [ProductType::DORIS] RINEX data
    pub fn doris(&self) -> Option<&Rinex> {
        self.data(ProductType::DORIS)?.as_rinex()
    }
    /// Returns reference to inner [ProductType::BroadcastNavigation] data
    pub fn brdc_navigation(&self) -> Option<&Rinex> {
        self.data(ProductType::BroadcastNavigation)?.as_rinex()
    }
    /// Returns reference to inner [ProductType::Meteo] data
    pub fn meteo(&self) -> Option<&Rinex> {
        self.data(ProductType::MeteoObservation)?.as_rinex()
    }
    /// Returns reference to inner [ProductType::HighPrecisionClock] data
    pub fn clock(&self) -> Option<&Rinex> {
        self.data(ProductType::HighPrecisionClock)?.as_rinex()
    }
    /// Returns reference to inner [ProductType::ANTEX] data
    pub fn antex(&self) -> Option<&Rinex> {
        self.data(ProductType::ANTEX)?.as_rinex()
    }
    /// Returns reference to inner [ProductType::IONEX] data
    pub fn ionex(&self) -> Option<&Rinex> {
        self.data(ProductType::IONEX)?.as_rinex()
    }
    /// Returns mutable reference to inner [ProductType::Observation] data
    pub fn observation_mut(&mut self) -> Option<&mut Rinex> {
        self.data_mut(ProductType::Observation)?.as_mut_rinex()
    }
    /// Returns mutable reference to inner [ProductType::DORIS] RINEX data
    pub fn doris_mut(&mut self) -> Option<&mut Rinex> {
        self.data_mut(ProductType::DORIS)?.as_mut_rinex()
    }
    /// Returns mutable reference to inner [ProductType::Observation] data
    pub fn brdc_navigation_mut(&mut self) -> Option<&mut Rinex> {
        self.data_mut(ProductType::BroadcastNavigation)?
            .as_mut_rinex()
    }
    /// Returns reference to inner [ProductType::Meteo] data
    pub fn meteo_mut(&mut self) -> Option<&mut Rinex> {
        self.data_mut(ProductType::MeteoObservation)?.as_mut_rinex()
    }
    /// Returns mutable reference to inner [ProductType::HighPrecisionClock] data
    pub fn clock_mut(&mut self) -> Option<&mut Rinex> {
        self.data_mut(ProductType::HighPrecisionClock)?
            .as_mut_rinex()
    }
    /// Returns mutable reference to inner [ProductType::HighPrecisionOrbit] data
    #[cfg(feature = "sp3")]
    #[cfg_attr(docrs, doc(cfg(feature = "sp3")))]
    pub fn sp3_mut(&mut self) -> Option<&mut SP3> {
        self.data_mut(ProductType::HighPrecisionOrbit)?.as_mut_sp3()
    }
    /// Returns mutable reference to inner [ProductType::ANTEX] data
    pub fn antex_mut(&mut self) -> Option<&mut Rinex> {
        self.data_mut(ProductType::ANTEX)?.as_mut_rinex()
    }
    /// Returns mutable reference to inner [ProductType::IONEX] data
    pub fn ionex_mut(&mut self) -> Option<&mut Rinex> {
        self.data_mut(ProductType::IONEX)?.as_mut_rinex()
    }
    /// Returns true if [ProductType::Observation] are present in Self
    pub fn has_observation(&self) -> bool {
        self.observation().is_some()
    }
    /// Returns true if [ProductType::BroadcastNavigation] are present in Self
    pub fn has_brdc_navigation(&self) -> bool {
        self.brdc_navigation().is_some()
    }
    #[cfg(feature = "sp3")]
    #[cfg_attr(docrs, doc(cfg(feature = "sp3")))]
    /// Returns true if [ProductType::HighPrecisionOrbit] are present in Self
    pub fn has_sp3(&self) -> bool {
        self.sp3().is_some()
    }
    /// Returns true if at least one [ProductType::DORIS] file is present
    pub fn has_doris(&self) -> bool {
        self.doris().is_some()
    }
    /// Returns true if [ProductType::MeteoObservation] are present in Self
    pub fn has_meteo(&self) -> bool {
        self.meteo().is_some()
    }
    #[cfg(feature = "sp3")]
    #[cfg_attr(docrs, doc(cfg(feature = "sp3")))]
    /// Returns true if High Precision Orbits also contains temporal information.
    pub fn sp3_has_clock(&self) -> bool {
        if let Some(sp3) = self.sp3() {
            sp3.sv_clock().count() > 0
        } else {
            false
        }
    }
    /// Load a single RINEX file into Self.
    /// File revision must be supported and must be correctly formatted
    /// for this operation to be effective.
    pub fn load_rinex(&mut self, path: &Path, rinex: Rinex) -> Result<(), Error> {
        let prod_type = ProductType::from(rinex.header.rinex_type);
        // extend context blob
        if let Some(paths) = self
            .files
            .iter_mut()
            .filter_map(|(prod, files)| {
                if *prod == prod_type {
                    Some(files)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
        {
            if let Some(inner) = self.blob.get_mut(&prod_type).and_then(|k| k.as_mut_rinex()) {
                inner.merge_mut(&rinex)?;
                paths.push(path.to_path_buf());
            }
        } else {
            self.blob.insert(prod_type, BlobData::Rinex(rinex));
            self.files.insert(prod_type, vec![path.to_path_buf()]);
        }
        Ok(())
    }
    /// Load a single SP3 file into Self.
    /// File revision must be supported and must be correctly formatted
    /// for this operation to be effective.
    #[cfg(feature = "sp3")]
    pub fn load_sp3(&mut self, path: &Path, sp3: SP3) -> Result<(), Error> {
        let prod_type = ProductType::HighPrecisionOrbit;
        // extend context blob
        if let Some(paths) = self
            .files
            .iter_mut()
            .filter_map(|(prod, files)| {
                if *prod == prod_type {
                    Some(files)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
        {
            if let Some(inner) = self.blob.get_mut(&prod_type).and_then(|k| k.as_mut_sp3()) {
                inner.merge_mut(&sp3)?;
                paths.push(path.to_path_buf());
            }
        } else {
            self.blob.insert(prod_type, BlobData::Sp3(sp3));
            self.files.insert(prod_type, vec![path.to_path_buf()]);
        }
        Ok(())
    }
    /// True if Self is compatible with navigation
    pub fn nav_compatible(&self) -> bool {
        self.observation().is_some() && self.brdc_navigation().is_some()
    }
    /// True if Self is compatible with CPP positioning,
    /// see <https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html#variant.CodePPP>
    pub fn cpp_compatible(&self) -> bool {
        if let Some(obs) = self.observation() {
            let mut prev_t = Option::<Epoch>::None;
            let mut prev_obs = HashMap::<SV, Vec<Observable>>::new();
            for ((t, _), sv, pr, _) in obs.pseudo_range() {
                if let Some(prev_t) = prev_t {
                    if prev_t != t {
                        prev_obs.clear();
                    }
                }
                if let Some(prev_obs) = prev_obs.get(&sv) {
                    if let Ok(first) = Carrier::from_observable(sv.constellation, pr) {
                        for ob in prev_obs {
                            if let Ok(second) = Carrier::from_observable(sv.constellation, ob) {
                                if second != first {
                                    return true;
                                }
                            }
                        }
                    }
                } else {
                    prev_obs.insert(sv, vec![pr.clone()]);
                }
                prev_t = Some(t);
            }
        }
        false
    }
    /// True if self is compatible with PPP positioning
    #[cfg(not(feature = "sp3"))]
    pub fn ppp_compatible(&self) -> bool {
        false
    }
    /// True if Self is compatible with CPP positioning,
    /// see <https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html#variant.PPP>
    #[cfg(feature = "sp3")]
    pub fn ppp_compatible(&self) -> bool {
        let has_dual_phase = if let Some(obs) = self.observation() {
            let mut compatible = false;
            let mut prev_t = Option::<Epoch>::None;
            let mut prev_obs = HashMap::<SV, Vec<Observable>>::new();
            for ((t, _), sv, pr, _) in obs.carrier_phase() {
                if compatible {
                    break;
                }
                if let Some(prev_t) = prev_t {
                    if prev_t != t {
                        prev_obs.clear();
                    }
                }
                if let Some(prev_obs) = prev_obs.get(&sv) {
                    if let Ok(first) = Carrier::from_observable(sv.constellation, pr) {
                        for ob in prev_obs {
                            if let Ok(second) = Carrier::from_observable(sv.constellation, ob) {
                                if second != first {
                                    compatible |= true;
                                }
                            }
                        }
                    }
                } else {
                    prev_obs.insert(sv, vec![pr.clone()]);
                }
                prev_t = Some(t);
            }
            compatible
        } else {
            false
        };
        self.clock().is_some() && self.sp3_has_clock() && self.cpp_compatible() && has_dual_phase
    }
    /// SP3 is require to 100% PPP compatibility
    #[cfg(not(feature = "sp3"))]
    pub fn ppp_ultra_compatible(&self) -> bool {
        false
    }
    #[cfg(feature = "sp3")]
    pub fn ppp_ultra_compatible(&self) -> bool {
        let same_timescale = if let Some(first_clk) = self.clock().and_then(|rnx| rnx.first_epoch())
        {
            if let Some(first_obs) = self.observation().and_then(|rnx| rnx.first_epoch()) {
                first_clk.time_scale == first_obs.time_scale
            } else {
                false
            }
        } else {
            false
        };
        self.ppp_compatible() && same_timescale
    }
    /// Returns true if provided Input products allow Ionosphere bias
    /// model optimization
    pub fn iono_bias_model_optimization(&self) -> bool {
        self.ionex().is_some() // TODO: BRDC V3 or V4
    }
    /// Returns true if provided Input products allow Troposphere bias
    /// model optimization
    pub fn tropo_bias_model_optimization(&self) -> bool {
        self.has_meteo()
    }
    /// Returns possible Reference position defined in this context.
    /// Usually the Receiver location in the laboratory.
    pub fn reference_position(&self) -> Option<GroundPosition> {
        if let Some(data) = self.observation() {
            if let Some(pos) = data.header.ground_position {
                return Some(pos);
            }
        }
        if let Some(data) = self.brdc_navigation() {
            if let Some(pos) = data.header.ground_position {
                return Some(pos);
            }
        }
        None
    }
    /// Apply preprocessing filter algorithm to mutable [Self].
    /// Filter will apply to all data contained in the context.
    pub fn filter_mut(&mut self, filter: &Filter) {
        if let Some(data) = self.observation_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.brdc_navigation_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.doris_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.meteo_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.clock_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.ionex_mut() {
            data.filter_mut(filter);
        }
        #[cfg(feature = "sp3")]
        if let Some(data) = self.sp3_mut() {
            data.filter_mut(filter);
        }
    }
    /// Fix given [Repair] condition
    pub fn repair_mut(&mut self, r: Repair) {
        if let Some(rinex) = self.observation_mut() {
            rinex.repair_mut(r);
        }
    }
}

impl std::fmt::Debug for QcContext {
    /// Debug formatting, prints all loaded files per Product category.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Primary: \"{}\"", self.name())?;
        for product in [
            ProductType::Observation,
            ProductType::BroadcastNavigation,
            ProductType::MeteoObservation,
            ProductType::HighPrecisionClock,
            ProductType::IONEX,
            ProductType::ANTEX,
            #[cfg(feature = "sp3")]
            ProductType::HighPrecisionOrbit,
        ] {
            if let Some(files) = self.files(product) {
                write!(f, "\n{}: ", product)?;
                write!(f, "{:?}", files,)?;
            }
        }
        Ok(())
    }
}
