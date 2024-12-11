//! User Data (input products) definitions
use thiserror::Error;

use std::{
    collections::HashMap,
    env,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use regex::Regex;

use rinex::{
    hardware::Receiver,
    prelude::{nav::Almanac, ParsingError as RinexParsingError, Rinex},
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

pub(crate) mod meta;
//pub(crate) mod navi;
//pub(crate) mod rtk;
pub(crate) mod clock;
pub(crate) mod iono;
pub(crate) mod meteo;
pub(crate) mod rnx;
pub(crate) mod session;
pub(crate) mod sky;
pub(crate) mod user_rover;

pub(crate) use meta::MetaData;

use crate::prelude::QcConfig;

use clock::ClockContext;
use iono::IonosphereContext;
use meteo::MeteoContext;
use sky::SkyContext;
use user_rover::UserRoverData;

/// Context Error
#[derive(Debug, Error)]
pub enum Error {
    #[error("i/o error")]
    IO,
    #[error("almanac error")]
    Alamanac(#[from] AlmanacError),
    #[error("planetary data error")]
    PlanetaryData(#[from] PlanetaryDataError),
    #[error("internal indexing/sorting issue")]
    DataIndexingIssue,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObservationUniqueId {
    /// GNSS [Receiver] is the prefered identifier
    Receiver(Receiver),
    /// Observer / Operator (not prefered)
    ObserverOperator(String),
}

impl std::fmt::LowerExp for ObservationUniqueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Receiver(rcvr) => write!(f, "{}", rcvr.model.to_lowercase()),
            Self::ObserverOperator(operator) => write!(f, "{}", operator.to_lowercase()),
        }
    }
}

impl std::fmt::UpperExp for ObservationUniqueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Receiver(rcvr) => write!(f, "{}", rcvr.model.to_uppercase()),
            Self::ObserverOperator(operator) => write!(f, "{}", operator.to_uppercase()),
        }
    }
}

/// [QcContext] is a general structure capable to store most common
/// GNSS data. It is dedicated to post processing workflows,
/// precise timing or atmosphere analysis.
pub struct QcContext {
    /// [QcConfig] used to deploy this [QcContext]
    pub cfg: QcConfig,
    /// Workspace is where this session will generate data.
    pub workspace: PathBuf,
    /// Latest Almanac to use during this session.
    pub almanac: Almanac,
    /// ECEF frame to use during this session. Based off [Almanac].
    pub earth_cef: Frame,
    /// [SkyContext] that applies worldwide / globally.
    sky_context: SkyContext,
    /// [MeteoContext] that either applies worldwide or regionally
    meteo_context: MeteoContext,
    /// [ClockContext] that either applies worldwide
    clk_context: ClockContext,
    /// [IonosphereContext] that either applies worldwide or regionally
    iono_context: IonosphereContext,
    /// Observation [Rinex] considered as the user observations,
    /// or "rover" in the context of RTK.
    user_rover_observations: Option<UserRoverData>,
    /// Reference or remote observations [Rinex], that enable
    /// differential analysis or RTK.
    reference_remote_observations: HashMap<ObservationUniqueId, Rinex>,
}

impl QcContext {
    /// ANISE storage location
    const ANISE_STORAGE_DIR: &str = "/home/env:USER/.local/share/nyx-space/anise";

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
    pub fn new<P: AsRef<Path>>(
        cfg: QcConfig,
        almanac: Almanac,
        frame: Frame,
        workspace: P,
    ) -> Result<Self, Error> {
        let s = Self {
            cfg,
            almanac,
            earth_cef: frame,
            sky_context: Default::default(),
            clk_context: Default::default(),
            iono_context: Default::default(),
            meteo_context: Default::default(),
            workspace: workspace.as_ref().to_path_buf(),
            user_rover_observations: Default::default(),
            reference_remote_observations: Default::default(),
        };

        s.deploy()?;
        Ok(s)
    }

    /// Build new [QcContext] to work in custom workspace.
    /// We will try to gather the daily JPL high precision data,
    /// which requires to access the internet once a day.
    /// If download fails, we rely on an offline model.
    pub fn new_daily_high_precision<P: AsRef<Path>>(
        cfg: QcConfig,
        workspace: P,
    ) -> Result<Self, Error> {
        let (almanac, earth_cef) = Self::build_almanac()?;
        Self::new(cfg, almanac, earth_cef, workspace)
    }

    // /// Returns possible Reference position defined in this context.
    // /// Usually the Receiver location in the laboratory.
    // pub fn user_rover_position(&self) -> Option<GroundPosition> {
    //     if let Some(data) = self.observation_data() {
    //         if let Some(pos) = data.header.ground_position {
    //             return Some(pos);
    //         }
    //     }
    //     if let Some(data) = self.brdc_navigation_data() {
    //         if let Some(pos) = data.header.ground_position {
    //             return Some(pos);
    //         }
    //     }
    //     None
    // }

    /// Smart data loader, that will automatically pick up the provided
    /// format (if supported) and load it into this [DataSet].
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let path = path.as_ref();
        let meta = MetaData::new(path)?;

        if let Ok(rinex) = Rinex::from_file(path) {
            self.load_rinex(meta, rinex);
            info!(
                "RINEX: \"{}\" loaded",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            return Ok(());
        }

        #[cfg(feature = "sp3")]
        if let Ok(sp3) = SP3::from_file(path) {
            self.sky_context.load_sp3(meta, sp3);
            info!(
                "SP3: \"{}\" loaded",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            return Ok(());
        }

        Err(Error::NonSupportedFormat)
    }

    /// Smart data loader, that will automatically pick up the provided
    /// format (if supported).
    #[cfg(feature = "flate2")]
    pub fn load_gzip_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let path = path.as_ref();
        let meta = MetaData::new(path)?;

        if let Ok(rinex) = Rinex::from_gzip_file(path) {
            self.load_rinex(meta, rinex);
            info!(
                "RINEX: \"{}\" loaded",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            return Ok(());
        }

        #[cfg(feature = "sp3")]
        if let Ok(sp3) = SP3::from_gzip_file(path) {
            self.sky_context.load_sp3(meta, sp3);
            info!(
                "SP3: \"{}\" loaded",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            return Ok(());
        }

        Err(Error::NonSupportedFormat)
    }

    /// Apply preprocessing filter algorithm to mutable [QcContext].
    /// This is an efficient interface to resample or shrink the input products.
    pub fn filter_mut(&mut self, filter: &Filter) {
        self.sky_context.filter_mut(&filter);
        self.meteo_context.filter_mut(&filter);
        self.iono_context.filter_mut(&filter);

        if let Some(data) = &mut self.user_rover_observations {
            data.data.filter_mut(&filter);
        }
    }

    /// "Fix" [QcContext] by applying [Repair]ment.
    pub fn repair_mut(&mut self, repair: Repair) {
        //self.sky_context.repair_mut(repair);
        self.meteo_context.repair_mut(repair);
        self.iono_context.repair_mut(repair);

        if let Some(rover_data) = &mut self.user_rover_observations {
            rover_data.data.repair_mut(repair);
        }
    }
}

impl std::fmt::Debug for QcContext {
    /// Debug formatting, prints all loaded files per Product category.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(rover) = &self.user_rover_observations {
            writeln!(f, "Observation (rover): \"{}\"", rover.meta.name)?;
        }

        Ok(())
    }
}
