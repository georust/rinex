//! User Data (input products) definitions
use thiserror::Error;

use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use regex::Regex;

use rinex::{
    prelude::{nav::Almanac, GroundPosition, ParsingError as RinexParsingError, Rinex, TimeScale},
    types::Type as RinexType,
};

use anise::{
    almanac::{metaload::MetaFile, planetary::PlanetaryDataError},
    constants::frames::{EARTH_ITRF93, IAU_EARTH_FRAME},
    errors::AlmanacError,
    prelude::Frame,
};

mod rinex_ctx;

#[cfg(feature = "sp3")]
#[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
mod sp3_ctx;

#[cfg(feature = "sp3")]
use sp3::prelude::SP3;

use qc_traits::{Filter, MergeError, Preprocessing, Repair, RepairTrait};

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
    RinexParsingError(#[from] RinexParsingError),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProductType {
    /// GNSS signal observations provided by Observation RINEX.
    Observation,
    /// Meteo observations provided by Meteo RINEX.
    MeteoObservation,
    /// DORIS signals observation provided by special RINEX.
    DORIS,
    /// Broadcast Navigation message described by Navigation RINEX.
    BroadcastNavigation,
    /// High precision clock states described by Clock RINEX.
    HighPrecisionClock,
    /// Antenna calibration information described by ANTEX.
    ANTEX,
    /// Precise Ionosphere maps described by IONEX.
    IONEX,
    #[cfg(feature = "sp3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
    /// High precision orbital attitude described by SP3.
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

impl ProductType {
    pub(crate) fn to_rinex_type(&self) -> Option<RinexType> {
        match self {
            Self::Observation => Some(RinexType::ObservationData),
            Self::ANTEX => Some(RinexType::AntennaData),
            Self::BroadcastNavigation => Some(RinexType::NavigationData),
            Self::DORIS => Some(RinexType::DORIS),
            Self::IONEX => Some(RinexType::IonosphereMaps),
            Self::MeteoObservation => Some(RinexType::MeteoData),
            Self::HighPrecisionClock => Some(RinexType::ClockData),
            #[cfg(feature = "sp3")]
            Self::HighPrecisionOrbit => None,
        }
    }
}

enum UserBlobData {
    /// RINEX content
    Rinex(Rinex),
    #[cfg(feature = "sp3")]
    /// SP3 content
    Sp3(SP3),
}

/// [UserData] provides input storage and classification
struct UserData {
    /// [UserBlobData]
    blob_data: UserBlobData,
    /// Files [PathBuf] that contributed, for this [InputKey].
    /// Stored in order of parsing.
    paths: Vec<PathBuf>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum UniqueId {
    /// GNSS receiver name/model differentiates a signal source
    Receiver(String),
    /// Data provider (agency) may differentiate some [ProductType]s
    Agency(String),
    /// A satellite may differentiate some [ProductType]s
    Satellite(String),
    /// Some [ProductType] are not differentiated
    #[default]
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct InputKey {
    /// [UniqueId] makes two identical [ProductType]s unique.
    pub unique_id: UniqueId,
    /// [ProductType] is the first level of uniqueness.
    pub product_type: ProductType,
}

/// [QcContext] is a general structure capable to store most common
/// GNSS data. It is dedicated to post processing workflows,
/// precise timing or atmosphere analysis.
pub struct QcContext {
    /// Complex [UserData] stored by [InputKey] that makes them unique
    user_data: HashMap<InputKey, UserData>,
    /// Latest Almanac to use during this session.
    pub almanac: Almanac,
    /// ECEF frame to use during this session. Based off [Almanac].
    pub earth_cef: Frame,
}

impl QcContext {
    /// ANISE storage location
    #[cfg(target_os = "linux")]
    const ANISE_STORAGE_DIR: &str = "/home/env:USER/.local/share/nyx-space/anise";

    /// ANISE storage location
    #[cfg(target_os = "macos")]
    const ANISE_STORAGE_DIR: &str = "/home/env:USER/.local/share/nyx-space/anise";

    /// ANISE storage location
    #[cfg(target_os = "windows")]
    const ANISE_STORAGE_DIR: &str = "C:/users/env:USER:/AppData/Local/nyx-space/anise";

    /// Helper to replace environment variables (if any)
    fn replace_env_vars(input: &str) -> String {
        let re = Regex::new(r"env:([A-Z_][A-Z0-9_]*)").unwrap();
        re.replace_all(input, |caps: &regex::Captures| {
            let var_name = &caps[1];
            env::var(var_name).unwrap_or_else(|_| format!("env:{}", var_name))
        })
        .to_string()
    }

    /// Returns [MetaFile] for anise DE440s.bsp
    fn nyx_anise_de440s_bsp() -> MetaFile {
        MetaFile {
            crc32: Some(1921414410),
            uri: String::from("http://public-data.nyxspace.com/anise/de440s.bsp"),
        }
    }

    /// Returns [MetaFile] for anise PCK11.pca
    fn nyx_anise_pck11_pca() -> MetaFile {
        MetaFile {
            crc32: Some(0x8213b6e9),
            uri: String::from("http://public-data.nyxspace.com/anise/v0.4/pck11.pca"),
        }
    }

    /// Returns [MetaFile] for daily JPL high precision bpc
    fn jpl_latest_high_prec_bpc() -> MetaFile {
        MetaFile {
            crc32: Self::jpl_latest_crc32(),
            uri:
                "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc"
                    .to_string(),
        }
    }

    /// Daily JPL high precision bpc CRC32 computation attempt.
    /// If file was previously downloaded, we return its CRC32.
    /// If it matches the CRC32 for today, download is ignored because it is not needed.
    fn jpl_latest_crc32() -> Option<u32> {
        let storage_dir = Self::replace_env_vars(Self::ANISE_STORAGE_DIR);
        let fullpath = format!("{}/earth_latest_high_prec.bpc", storage_dir);

        if let Ok(mut fd) = File::open(fullpath) {
            let mut buf = Vec::with_capacity(1024);
            match fd.read_to_end(&mut buf) {
                Ok(_) => Some(crc32fast::hash(&buf)),
                Err(_) => None,
            }
        } else {
            None
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
            user_data: Default::default(),
        })
    }

    /// Build new [QcContext] with given [Almanac] and desired [Frame],
    /// which must be one of the available ECEF.
    pub fn new_almanac(almanac: Almanac, frame: Frame) -> Result<Self, Error> {
        Ok(Self {
            almanac,
            earth_cef: frame,
            user_data: Default::default(),
        })
    }

    /// Returns general [TimeScale] for this [QcContext]
    pub fn timescale(&self) -> Option<TimeScale> {
        #[cfg(feature = "sp3")]
        if let Some(sp3) = self.sp3_data() {
            return Some(sp3.time_scale);
        }

        if let Some(obs) = self.observation_data() {
            let first = obs.first_epoch()?;
            Some(first.time_scale)
        } else if let Some(dor) = self.doris_data() {
            let first = dor.first_epoch()?;
            Some(first.time_scale)
        } else if let Some(clk) = self.clock_data() {
            let first = clk.first_epoch()?;
            Some(first.time_scale)
        } else if self.meteo_data().is_some() {
            Some(TimeScale::UTC)
        } else if self.ionex_data().is_some() {
            Some(TimeScale::UTC)
        } else {
            None
        }
    }

    /// Returns path to File considered as Primary input product.
    /// If [QcContext] only holds one input product, it is obviously defined as Primary,
    /// whatever its kind.
    pub fn primary_path(&self) -> Option<&Path> {
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
            // Returns first file loaded in this category.
            if let Some(first) = self.files_iter(Some(product), None).next() {
                return Some(first);
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

    /// Returns local file names Iterator.
    /// Use [ProductType] filter to restrict to specific input [ProductType].
    /// Use [UniqueId] filter to restrict to specific data source.
    pub fn files_iter(
        &self,
        product_id: Option<ProductType>,
        source_id: Option<UniqueId>,
    ) -> Box<dyn Iterator<Item = &Path> + '_> {
        if let Some(product_id) = product_id {
            if let Some(source_id) = source_id {
                Box::new(
                    self.user_data
                        .iter()
                        .filter_map(move |(k, v)| {
                            if k.product_type == product_id {
                                if k.unique_id == source_id {
                                    Some(v.paths.iter().map(|k| k.as_path()))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .flatten(),
                )
            } else {
                Box::new(
                    self.user_data
                        .iter()
                        .filter_map(move |(k, v)| {
                            if k.product_type == product_id {
                                Some(v.paths.iter().map(|k| k.as_path()))
                            } else {
                                None
                            }
                        })
                        .flatten(),
                )
            }
        } else {
            if let Some(source_id) = source_id {
                Box::new(
                    self.user_data
                        .iter()
                        .filter_map(move |(k, v)| {
                            if k.unique_id == source_id {
                                Some(v.paths.iter().map(|k| k.as_path()))
                            } else {
                                None
                            }
                        })
                        .flatten(),
                )
            } else {
                Box::new(
                    self.user_data
                        .iter()
                        .flat_map(|(_, v)| v.paths.iter().map(|k| k.as_path())),
                )
            }
        }
    }

    /// Returns [UserData] for this [ProductType]
    fn get_per_product_user_data(&self, product_id: ProductType) -> Option<&UserData> {
        self.user_data
            .iter()
            .filter_map(|(k, user_data)| {
                if k.product_type == product_id {
                    Some(user_data)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }

    /// Returns mutable [UserData] for this [ProductType]
    fn get_per_product_user_data_mut(&mut self, product_id: ProductType) -> Option<&mut UserData> {
        self.user_data
            .iter_mut()
            .filter_map(|(k, user_data)| {
                if k.product_type == product_id {
                    Some(user_data)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }

    /// Returns [UserData] for this unique [InputKey] combination
    fn get_unique_user_data(&self, key: &InputKey) -> Option<&UserData> {
        self.user_data
            .iter()
            .filter_map(
                |(k, user_data)| {
                    if k == key {
                        Some(user_data)
                    } else {
                        None
                    }
                },
            )
            .reduce(|k, _| k)
    }

    /// Returns [UserData] for this unique [InputKey] combination
    fn get_unique_user_data_mut(&mut self, key: &InputKey) -> Option<&mut UserData> {
        self.user_data
            .iter_mut()
            .filter_map(
                |(k, user_data)| {
                    if k == key {
                        Some(user_data)
                    } else {
                        None
                    }
                },
            )
            .reduce(|k, _| k)
    }

    /// True if [QcContext] is compatible with post processed navigation
    pub fn is_navi_compatible(&self) -> bool {
        self.observation_data().is_some() && self.brdc_navigation_data().is_some()
    }

    /// True if [QcContext] is compatible with CPP positioning,
    /// see <https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html#variant.CodePPP>
    pub fn is_cpp_compatible(&self) -> bool {
        // TODO: improve: only PR
        if let Some(obs) = self.observation_data() {
            obs.carrier_iter().count() > 1
        } else {
            false
        }
    }

    /// True if [QcContext] is compatible with PPP positioning,
    /// see <https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html#variant.PPP>
    pub fn is_ppp_compatible(&self) -> bool {
        // TODO: check PH as well
        self.is_cpp_compatible()
    }

    #[cfg(not(feature = "sp3"))]
    /// SP3 support is required for 100% PPP compatibility
    pub fn ppp_ultra_compatible(&self) -> bool {
        false
    }

    #[cfg(feature = "sp3")]
    pub fn is_ppp_ultra_compatible(&self) -> bool {
        // TODO: improve
        //      verify clock().ts and obs().ts do match
        //      and have common time frame
        self.clock_data().is_some() && self.sp3_has_clock() && self.is_ppp_compatible()
    }

    /// True if [QcContext] supports Ionosphere bias model optimization
    pub fn allows_iono_bias_model_optimization(&self) -> bool {
        self.ionex_data().is_some() // TODO: BRDC V3 or V4
    }

    /// True if [QcContext] supports Troposphere bias model optimization
    pub fn allows_tropo_bias_model_optimization(&self) -> bool {
        self.has_meteo()
    }

    /// Returns possible Reference position defined in this context.
    /// Usually the Receiver location in the laboratory.
    pub fn reference_position(&self) -> Option<GroundPosition> {
        if let Some(data) = self.observation_data() {
            if let Some(pos) = data.header.ground_position {
                return Some(pos);
            }
        }
        if let Some(data) = self.brdc_navigation_data() {
            if let Some(pos) = data.header.ground_position {
                return Some(pos);
            }
        }
        None
    }

    /// Apply preprocessing filter algorithm to mutable [QcContext].
    /// This is an efficient interface to resample or shrink the input products.
    pub fn filter_mut(&mut self, filter: &Filter) {
        if let Some(data) = self.observation_data_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.brdc_navigation_data_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.doris_data_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.meteo_data_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.clock_data_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.ionex_data_mut() {
            data.filter_mut(filter);
        }
        #[cfg(feature = "sp3")]
        if let Some(data) = self.sp3_data_mut() {
            data.filter_mut(filter);
        }
    }

    /// "Fix" [QcContext] by applyaing [Repair]ment.
    /// This is useful if some wrongfuly Null Observations were forwarded
    /// and we need to patch them, and similar scenarios.
    pub fn repair_mut(&mut self, r: Repair) {
        if let Some(rinex) = self.observation_data_mut() {
            rinex.repair_mut(r);
        }
        if let Some(rinex) = self.meteo_data_mut() {
            rinex.repair_mut(r);
        }
        if let Some(rinex) = self.doris_data_mut() {
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
            while let Some(path) = self.files_iter(Some(product), None).next() {
                write!(f, "\n{}: ", product)?;
                write!(f, "{:?}", path)?;
            }
        }
        Ok(())
    }
}
