//! User Data (input products) definitions
use thiserror::Error;

use std::{
    collections::{hash_map::Keys, HashMap},
    env,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use regex::Regex;

use rinex::prelude::{
    nav::{Almanac, Orbit},
    GroundPosition, ParsingError as RinexParsingError, Rinex, TimeScale,
};

use anise::{
    almanac::{metaload::MetaFile, planetary::PlanetaryDataError},
    constants::frames::{EARTH_ITRF93, IAU_EARTH_FRAME},
    errors::AlmanacError,
    prelude::Frame,
};

use qc_traits::{Filter, MergeError, Preprocessing, Repair, RepairTrait};

#[cfg(feature = "sp3")]
use sp3::prelude::SP3;

// Context post processing.
// This that can only be achieved by stacking more than one RINEX
// and possibly one SP3.
mod processing;

pub(crate) mod dataset;
pub(crate) mod session;

use dataset::DataSet;

/// Context Error
#[derive(Debug, Error)]
pub enum Error {
    #[error("i/o error")]
    IO,
    #[error("almanac error")]
    Alamanac(#[from] AlmanacError),
    #[error("planetary data error")]
    PlanetaryData(#[from] PlanetaryDataError),
    #[error("failed to extend gnss context")]
    ContextExtensionError(#[from] MergeError),
    #[error("non supported file format")]
    NonSupportedFormat,
    #[error("failed to determine file name")]
    FileName,
    #[error("failed to determine file extension")]
    FileExtension,
    #[error("invalid rinex format")]
    RinexParsingError(#[from] RinexParsingError),
}

/// [QcContext] is a general structure capable to store most common
/// GNSS data. It is dedicated to post processing workflows,
/// precise timing or atmosphere analysis.
pub struct QcContext {
    /// Workspace is where this session will generate data.
    pub workspace: PathBuf,
    /// Latest Almanac to use during this session.
    pub almanac: Almanac,
    /// ECEF frame to use during this session. Based off [Almanac].
    pub earth_cef: Frame,
    /// Global data that may apply to both [DataSet] and enhanced capabilities.
    /// This type of data is usually valid worlwide
    global_data: GlobalDataSet,
    /// [DataSet] that either represents the User main data or the rover 
    /// in differential analysis.
    rover_dataset: DataSet,
    /// Possible remote [DataSet] that enables differential analysis
    remote_dataset: DataSet,
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

    /// Creates a new [QcContext] with specified [Almanac]
    /// and reference [Frame] that must be Earth centered.
    /// The session will generate data within desired workspace.
    pub fn new<P: AsRef<Path>>(almanac: Almanac, frame: Frame, workspace: P) -> Result<Self, Error> {
        let s = Self {
            almanac,
            earth_cef: frame,
            rover_dataset: Default::default(),
            remote_dataset: Default::default(),
            workspace: workspace.as_ref().to_path_buf(),
        };

        s.deploy()?;
        Ok(s)
    }

    /// Build new [QcContext] to work in custom workspace.
    /// We will try to gather the daily JPL high precision data,
    /// which requires to access the internet once a day.
    /// If download fails, we rely on an offline model.
    pub fn new_daily_high_precision<P: AsRef<Path>>(workspace: P) -> Result<Self, Error> {
        let (almanac, earth_cef) = Self::build_almanac()?;
        Self::new(almanac, earth_cef, workspace)
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
    pub fn user_rover_position(&self) -> Option<GroundPosition> {
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

    /// Returns true if [QcContext] is RTK compatible
    pub fn is_rtk_compatible(&self) -> bool {
        !self.data_context.is_empty()
        && !self.remote_context.is_empty()
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

    /// Returns possible remote positions
    pub fn remote_position(&self) -> Option<GroundPosition> {

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
        if !self.single_file() {
            writeln!(f, "Context is: \"{}\"", self.primary_name().unwrap())?;
        }

        for product in [
            ProductType::Observation,
            ProductType::BroadcastNavigation,
            ProductType::MeteoObservation,
            ProductType::HighPrecisionClock,
            ProductType::IONEX,
            ProductType::ANTEX,
        ] {
            for meta in self.meta_iter().filter(|meta| meta.product_id == product) {
                writeln!(f, "{}: \"{}\"", meta.product_id, meta.name)?;
            }
        }

        #[cfg(feature = "sp3")]
        for meta in self
            .meta_iter()
            .filter(|meta| meta.product_id == ProductType::HighPrecisionOrbit)
        {
            writeln!(f, "{}: \"{}\"", meta.product_id, meta.name)?;
        }

        Ok(())
    }
}
