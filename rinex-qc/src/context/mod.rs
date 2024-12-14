//! User Data (input products) definitions
use thiserror::Error;

use std::{
    collections::HashMap,
    env,
    fs::create_dir_all,
    fs::File,
    io::Read,
    io::Write,
    path::{Path, PathBuf},
};

use regex::Regex;

use rinex::{
    hardware::Receiver,
    prelude::{nav::Almanac, ParsingError as RinexParsingError, Rinex},
};

use anise::{
    almanac::{
        metaload::{MetaAlmanac, MetaAlmanacError, MetaFile},
        planetary::PlanetaryDataError,
    },
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
    #[error("alamanc setup issue: {0}")]
    MetaAlamanac(#[from] MetaAlmanacError),
    #[error("frame model error: {0}")]
    FrameModel(#[from] PlanetaryDataError),
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
    const ANISE_ALMANAC_STORAGE: &str = ".cache";

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
    fn nyx_anise_jpl_bpc() -> MetaFile {
        MetaFile {
            crc32: None,
            uri:
                "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc"
                    .to_string(),
        }
    }

    /// Method to either download, retrieve or create
    /// a basic [Almanac] and reference [Frame] to work with.
    /// This will try to download the highest JPL model, and requires
    /// internet access once a day.
    /// If the JPL database cannot be accessed, we rely on an offline model.
    fn build_almanac_frame_model() -> Result<(Almanac, Frame), Error> {
        let mut initial_setup = false;

        // Meta almanac for local storage management
        let local_storage = format!(
            "{}/{}/anise.dhall",
            env!("CARGO_MANIFEST_DIR"),
            Self::ANISE_ALMANAC_STORAGE
        );

        let mut meta_almanac = match MetaAlmanac::new(local_storage.clone()) {
            Ok(meta) => {
                debug!("(anise) from local storage");
                meta
            },
            Err(_) => {
                debug!("(anise) local storage setup");
                initial_setup = true;
                MetaAlmanac {
                    files: vec![
                        Self::nyx_anise_de440s_bsp(),
                        Self::nyx_anise_pck11_pca(),
                        Self::nyx_anise_jpl_bpc(),
                    ],
                }
            },
        };

        // download (if need be)
        let almanac = meta_almanac.process(true)?;

        if initial_setup {
            let updated = meta_almanac.dumps()?;

            let _ = create_dir_all(&format!(
                "{}/{}",
                env!("CARGO_MANIFEST_DIR"),
                Self::ANISE_ALMANAC_STORAGE
            ));

            let mut fd = File::create(&local_storage)
                .unwrap_or_else(|e| panic!("almanac storage setup error: {}", e));

            fd.write_all(updated.as_bytes())
                .unwrap_or_else(|e| panic!("almanac storage setup error: {}", e));
        }

        match almanac.frame_from_uid(EARTH_ITRF93) {
            Ok(itrf93) => {
                info!("highest precision context setup");
                return Ok((almanac, itrf93));
            },
            Err(e) => {
                error!("(anise) jpl_bpc: {}", e);
            },
        }

        let earth_cef = almanac.frame_from_uid(IAU_EARTH_FRAME)?;
        warn!("deployed with offline model");
        Ok((almanac, earth_cef))
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
        let (almanac, earth_cef) = Self::build_almanac_frame_model()?;
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
            self.sky_context.load_sp3(meta, sp3)?;
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
