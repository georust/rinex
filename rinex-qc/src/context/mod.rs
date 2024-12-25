use std::{collections::HashMap, env, fs::create_dir_all, fs::File, io::Write, path::Path};

use rinex::prelude::{nav::Almanac, Rinex};

use anise::{
    almanac::metaload::{MetaAlmanac, MetaFile},
    constants::frames::{EARTH_ITRF93, IAU_EARTH_FRAME},
    prelude::Frame,
};

use qc_traits::{Filter, Preprocessing, Repair, RepairTrait};

// Context post processing.
// This that can only be achieved by stacking more than one RINEX
// and possibly one SP3.
mod processing;

pub(crate) mod ionex;
pub(crate) mod meta;
pub(crate) mod meteo;
pub(crate) mod nav;
pub(crate) mod obs;
pub(crate) mod report;
pub(crate) mod rnx;
pub(crate) mod session;
pub(crate) mod tropo;

use crate::{
    cfg::{QcConfig, QcFrameModel},
    context::{
        // clock::ClockDataSet,
        // sky::SkyContext,
        meta::MetaData,
    },
    report::QcReport,
    QcCtxError,
};

/// [QcContext] is a general structure capable to store most common
/// GNSS data. It is dedicated to post processing workflows,
/// precise timing or atmosphere analysis.
pub struct QcContext {
    /// [QcConfig] used to deploy this [QcContext]
    pub cfg: QcConfig,
    /// Latest Almanac to use during this session.
    pub almanac: Almanac,
    /// ECEF frame to use during this session. Based off [Almanac].
    pub earth_cef: Frame,
    /// Observation [Rinex] stored by [MetaData]
    pub obs_dataset: HashMap<MetaData, Rinex>,
    /// Possible Navigation [Rinex]
    pub nav_dataset: Option<Rinex>,
    /// Possible IONEx [Rinex]
    pub ionex_dataset: Option<Rinex>,
    /// Meteo [Rinex] stored by [MetaData]
    pub meteo_dataset: HashMap<MetaData, Rinex>,
    // pub(crate) sky_context: Option<SkyContext>,
    // pub(crate) clk_dataset: Option<ClockDataSet>,
    // /// [MeteoContext] that either applies regionally or worldwidely
    // meteo_context: MeteoContext,
    // /// [ClockContext] that either applies worldwidely
    // clk_context: ClockContext,
    // /// [IonosphereContext] that either applies regionally or worldwidely
    // iono_context: IonosphereContext,
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
    fn build_almanac_frame_model(prefered: QcFrameModel) -> Result<(Almanac, Frame), QcCtxError> {
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

        if prefered == QcFrameModel::ITRF93 {
            // try to form the EARTH ITRF93 frame model
            match almanac.frame_from_uid(EARTH_ITRF93) {
                Ok(itrf93) => {
                    info!("earth_itrf93 frame model loaded");
                    return Ok((almanac, itrf93));
                },
                Err(e) => {
                    error!("(anise) itrf93: {}", e);
                },
            }
        }

        let earth_cef = almanac.frame_from_uid(IAU_EARTH_FRAME)?;
        warn!("deployed with offline model");
        Ok((almanac, earth_cef))
    }

    /// Creates a new [QcContext] with [QcConfig] configuration preset.
    pub fn new(cfg: QcConfig) -> Result<Self, QcCtxError> {
        let mut cfg = cfg.clone();

        let (almanac, earth_cef) = Self::build_almanac_frame_model(cfg.navi.frame_model)?;

        if earth_cef == EARTH_ITRF93 {
            cfg.navi.frame_model = QcFrameModel::ITRF93;
        } else {
            cfg.navi.frame_model = QcFrameModel::IAU;
        }

        let s = Self {
            cfg,
            almanac,
            earth_cef,
            obs_dataset: Default::default(),
            nav_dataset: Default::default(),
            meteo_dataset: Default::default(),
            ionex_dataset: Default::default(),
        };

        s.deploy()?;
        Ok(s)
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
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), QcCtxError> {
        let path = path.as_ref();
        let mut meta = MetaData::new(path)?;

        if let Ok(rinex) = Rinex::from_file(path) {
            self.load_rinex(&mut meta, rinex)?;
            info!(
                "{} (RINex) loaded",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            return Ok(());
        }

        // #[cfg(feature = "sp3")]
        // if let Ok(sp3) = SP3::from_file(path) {
        //     self.sky_context.load_sp3(meta, sp3)?;
        //     info!(
        //         "{} (SP3) loaded",
        //         path.file_stem().unwrap_or_default().to_string_lossy()
        //     );
        //     return Ok(());
        // }

        Err(QcCtxError::NonSupportedFormat)
    }

    /// Smart data loader, that will automatically pick up the provided
    /// format (if supported).
    #[cfg(feature = "flate2")]
    pub fn load_gzip_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), QcCtxError> {
        let path = path.as_ref();
        let mut meta = MetaData::new(path)?;

        if let Ok(rinex) = Rinex::from_gzip_file(path) {
            self.load_rinex(&mut meta, rinex)?;
            info!(
                "{} (RINex) loaded",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            return Ok(());
        }

        // #[cfg(feature = "sp3")]
        // if let Ok(sp3) = SP3::from_gzip_file(path) {
        //     self.sky_context.load_sp3(meta, sp3);
        //     info!(
        //         "SP3: \"{}\" loaded",
        //         path.file_stem().unwrap_or_default().to_string_lossy()
        //     );
        //     return Ok(());
        // }

        Err(QcCtxError::NonSupportedFormat)
    }

    pub fn filter_mut(&mut self, filter: &Filter) {
        for (_, rinex) in self.obs_dataset.iter_mut() {
            rinex.filter_mut(&filter);
        }
        if let Some(rinex) = &mut self.nav_dataset {
            rinex.filter_mut(&filter);
        }
        if let Some(rinex) = &mut self.ionex_dataset {
            rinex.filter_mut(&filter);
        }
        for (_, rinex) in self.meteo_dataset.iter_mut() {
            rinex.filter_mut(&filter);
        }
    }

    pub fn repair_mut(&mut self, repair: Repair) {
        for (_, rinex) in self.obs_dataset.iter_mut() {
            rinex.repair_mut(repair);
        }
    }

    /// Synthesize a [QcReport], performs analysis
    /// and calculations to fill the report according to [QcConfig] preset.
    /// Report synthesis is not rendition! You need to use the
    /// rendition trait when you want to.
    pub fn report_synthesis(&self) -> QcReport {
        QcReport::new(self)
    }
}

impl std::fmt::Debug for QcContext {
    /// Debug formatting, prints all loaded files per Product category.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, _) in &self.obs_dataset {
            write!(f, "Observation RINex: {}", k.name)?;
        }
        Ok(())
    }
}
