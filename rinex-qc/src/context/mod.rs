//! GNSS processing context definition.
use thiserror::Error;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use rinex::{
    merge::Error as RinexMergeError,
    prelude::{Almanac, GroundPosition, Rinex, TimeScale},
    types::Type as RinexType,
    Error as RinexError,
};

use anise::{
    almanac::{metaload::MetaFile, planetary::PlanetaryDataError},
    constants::frames::{EARTH_ITRF93, IAU_EARTH_FRAME},
    errors::AlmanacError,
    prelude::Frame,
};

mod rinex_ctx;
mod iter;

#[cfg(feature = "sp3")]
#[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
mod sp3_ctx;

#[cfg(feature = "sp3")]
use sp3::prelude::SP3;

use qc_traits::{
    processing::{Filter, Preprocessing, Repair, RepairTrait},
    MergeError,
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
    #[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
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
    /// ECEF frame
    pub earth_cef: Frame,
}

impl QcContext {
    /// DE400s BSP can be stored forever
    fn nyx_anise_de440s_bsp() -> MetaFile {
        MetaFile {
            crc32: Some(1921414410),
            uri: String::from("http://public-data.nyxspace.com/anise/de440s.bsp"),
        }
    }
    /// PCK11 PCA can be stored forever
    fn nyx_anise_pck11_pca() -> MetaFile {
        MetaFile {
            crc32: Some(0x8213b6e9),
            uri: String::from("http://public-data.nyxspace.com/anise/v0.4/pck11.pca"),
        }
    }
    /// JPL latest requires at best, a daily download
    fn jpl_latest_high_prec_bpc() -> MetaFile {
        MetaFile {
            crc32: None,
            uri:
                "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc"
                    .to_string(),
        }
    }
    /// Infaillible method to either download, retrieve or create
    /// a basic [Almanac] and reference [Frame] to work with.
    /// We always prefer the highest precision scenario.
    /// On first deployment, it will require internet access.
    /// We can only rely on lower precision kernels if we cannot access the cloud.
    fn build_almanac() -> Result<(Almanac, Frame), Error> {
        let almanac = Almanac::until_2035()?;
        match almanac.load_from_metafile(Self::nyx_anise_de440s_bsp()) {
            Ok(almanac) => {
                info!("ANISE DE440S BSP has been loaded");
                match almanac.load_from_metafile(Self::nyx_anise_pck11_pca()) {
                    Ok(almanac) => {
                        info!("ANISE PCK11 PCA has been loaded");
                        match almanac.load_from_metafile(Self::jpl_latest_high_prec_bpc()) {
                            Ok(almanac) => {
                                info!("JPL high precision (daily) kernels loaded.");
                                if let Ok(itrf93) = almanac.frame_from_uid(EARTH_ITRF93) {
                                    info!("Deployed with highest precision context available.");
                                    Ok((almanac, itrf93))
                                } else {
                                    let iau_earth = almanac.frame_from_uid(IAU_EARTH_FRAME)?;
                                    warn!("Failed to build ITRF93: relying on IAU model");
                                    Ok((almanac, iau_earth))
                                }
                            },
                            Err(e) => {
                                let iau_earth = almanac.frame_from_uid(IAU_EARTH_FRAME)?;
                                error!("Failed to download JPL High precision kernels: {}", e);
                                warn!("Relying on IAU frame model.");
                                Ok((almanac, iau_earth))
                            },
                        }
                    },
                    Err(e) => {
                        let iau_earth = almanac.frame_from_uid(IAU_EARTH_FRAME)?;
                        error!("Failed to download PCK11 PCA: {}", e);
                        warn!("Relying on IAU frame model.");
                        Ok((almanac, iau_earth))
                    },
                }
            },
            Err(e) => {
                error!("Failed to load DE440S BSP: {}", e);
                let iau_earth = almanac.frame_from_uid(IAU_EARTH_FRAME)?;
                warn!("Relying on IAU frame model.");
                Ok((almanac, iau_earth))
            },
        }
    }
    /// Create a new QcContext for which we will try to
    /// retrieve the latest and highest precision [Almanac]
    /// and reference [Frame] to work with. If you prefer
    /// to manualy specify those, prefer the other constructor.
    pub fn new() -> Result<Self, Error> {
        let (almanac, earth_cef) = Self::build_almanac()?;
        Ok(Self {
            almanac,
            earth_cef,
            blob: HashMap::new(),
            files: HashMap::new(),
        })
    }
    /// Build new [QcContext] with given [Almanac] and desired [Frame],
    /// which must be one of the available ECEF.
    pub fn new_almanac(almanac: Almanac, frame: Frame) -> Result<Self, Error> {
        Ok(Self {
            almanac,
            earth_cef: frame,
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

    /// True if [QcContext]] is compatible with post processed navigation
    pub fn nav_compatible(&self) -> bool {
        self.observation().is_some() && self.brdc_navigation().is_some()
    }

    /// Returns True if [QcContext] is compatible with CPP positioning,
    /// as per <https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html#variant.CodePPP>
    pub fn cpp_compatible(&self) -> bool {
        // TODO: improve: only PR
        if let Some(obs) = self.observation() {
            obs.carrier().count() > 1
        } else {
            false
        }
    }

    /// Returns True if [QcContext] is compatible with PPP positioning,
    /// as per <https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html#variant.PPP>
    pub fn ppp_compatible(&self) -> bool {
        // TODO: check PH as well
        self.cpp_compatible()
    }

    #[cfg(not(feature = "sp3"))]
    /// SP3 is required for 100% PPP compatibility
    pub fn ppp_ultra_compatible(&self) -> bool {
        false
    }

    #[cfg(feature = "sp3")]
    pub fn ppp_ultra_compatible(&self) -> bool {
        // TODO: improve
        //      verify clock().ts and obs().ts do match
        //      and have common time frame
        self.clock().is_some() && self.sp3_has_clock() && self.ppp_compatible()
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
