//! GNSS processing context definition.
use thiserror::Error;

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::create_dir_all,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use rinex::{
    merge::{Error as RinexMergeError, Merge as RinexMerge},
    prelude::{Almanac, GroundPosition, Rinex, TimeScale},
    types::Type as RinexType,
    Error as RinexError,
};

use anise::{
    almanac::{
        metaload::MetaAlmanacError,
        metaload::{MetaAlmanac, MetaFile},
        planetary::PlanetaryDataError,
    },
    constants::frames::{EARTH_ITRF93, IAU_EARTH_FRAME},
    errors::AlmanacError,
    prelude::Frame,
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
    #[error("almanac error: {0}")]
    Almanac(#[from] AlmanacError),
    #[error("meta error: {0}")]
    MetaAlmanac(#[from] MetaAlmanacError),
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
    #[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
    pub fn as_sp3(&self) -> Option<&SP3> {
        match self {
            Self::Sp3(s) => Some(s),
            _ => None,
        }
    }
    /// Returns mutable reference to inner SP3 data.
    #[cfg(feature = "sp3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
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
    /// ECEF frame
    pub earth_cef: Frame,
}

impl QcContext {
    const ALMANAC_LOCAL_STORAGE: &str = ".cache";

    fn nyx_anise_de440s_bsp() -> MetaFile {
        MetaFile {
            crc32: Some(0x7286750a),
            uri: String::from("http://public-data.nyxspace.com/anise/de440s.bsp"),
        }
    }

    fn nyx_anise_pck11_pca() -> MetaFile {
        MetaFile {
            crc32: Some(0x8213b6e9),
            uri: String::from("http://public-data.nyxspace.com/anise/v0.5/pck11.pca"),
        }
    }

    fn nyx_anise_jpl_bpc() -> MetaFile {
        MetaFile {
            crc32: None,
            uri:
                "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc"
                    .to_string(),
        }
    }

    /// This [MetaAlmanac] solely relies on the nyx-space servers
    fn low_prec_meta() -> MetaAlmanac {
        MetaAlmanac {
            files: vec![Self::nyx_anise_pck11_pca(), Self::nyx_anise_de440s_bsp()],
        }
    }

    /// This [MetaAlmanac] relies on the nyx-space servers
    /// and the daily JPL update
    fn high_prec_meta() -> MetaAlmanac {
        MetaAlmanac {
            files: vec![
                Self::nyx_anise_de440s_bsp(),
                Self::nyx_anise_pck11_pca(),
                Self::nyx_anise_jpl_bpc(),
            ],
        }
    }

    /// Creates a new [QcContext].
    /// Requires network access on first deployment ever (otherwise will fail)
    /// because initial deployment will initiate a local storage
    /// which makes future deployments much faster.
    ///
    /// ## Input
    /// - "jpl_update" == false
    /// When "update is set to false, this session will rely either on:
    /// - previously downloaded high precision model
    /// - previously downloaded low precision model (if "update" has never been used)
    ///
    /// - "jpl_update" == true
    /// Each call will require valid internet access otherwise will fail.
    /// We attempt the upgrade (or initial setup) of the highest precision model.
    /// The model is valid for up to 3 months, in practice you can safely use it
    /// for a couple of days or weeks, until network is available for a new update.
    pub fn new(jpl_update: bool) -> Result<Self, Error> {
        let local_storage = format!(
            "{}/{}/anise.dhall",
            env!("CARGO_MANIFEST_DIR"),
            Self::ALMANAC_LOCAL_STORAGE
        );

        // if the "".dhall"" doesnt exist, this is first deployment ever
        let anise_exist = Path::new(&local_storage).exists();

        let mut uses_jpl_model = false;

        let almanac = if !anise_exist {
            // prepare storage
            create_dir_all(&format!(
                "{}/{}",
                env!("CARGO_MANIFEST_DIR"),
                Self::ALMANAC_LOCAL_STORAGE
            ))
            .unwrap_or_else(|e| panic!("almanac storage setup error: {}", e));

            let almanac = Self::first_almanac_ever(&local_storage, jpl_update)?;
            uses_jpl_model = true;
            almanac
        } else {
            // using local storage
            let mut meta = match MetaAlmanac::new(local_storage.clone()) {
                Ok(meta) => meta,
                Err(e) => {
                    error!("(anise) failed to retrieve local storage!");
                    return Err(Error::MetaAlmanac(e));
                },
            };

            // force JPL update
            if jpl_update {
                for file in &mut meta.files {
                    if file.uri.contains("earth_latest_high_prec.bpc") {
                        file.crc32 = None;
                        uses_jpl_model = true;
                        break;
                    }
                }

                if !uses_jpl_model {
                    meta.files.push(Self::nyx_anise_jpl_bpc());
                }
            }

            match meta.process(true) {
                Ok(almanac) => {
                    if jpl_update {
                        info!("(anise) jpl latest high precision model updated");

                        // try to save for later (upgrading the local storage)
                        match meta.dumps() {
                            Ok(serialized) => {
                                let mut fd = File::create(&local_storage).unwrap_or_else(|e| {
                                    panic!("almanac storage setup error: {}", e)
                                });

                                fd.write_all(serialized.as_bytes()).unwrap_or_else(|e| {
                                    panic!("almanac storage setup error: {}", e)
                                });
                            },
                            Err(e) => {
                                error!("(anise) almanac serialization failed: {}", e)
                            },
                        }
                    } else {
                        info!("(anise) using previous model");
                    }
                    almanac
                },
                Err(e) => {
                    // upgrade failed
                    error!("(anise) context setup error: {}", e);
                    return Err(Error::Almanac(e));
                },
            }
        };

        // always try to obtain the ITRF93 frame
        // which succeeds on ideal scenario, and may fail
        // depending on deployment status
        let earth_cef = if uses_jpl_model {
            match almanac.frame_from_uid(EARTH_ITRF93) {
                Ok(ecef) => {
                    info!("(anise) deployed with high precision model");
                    ecef
                },
                Err(e) => {
                    error!("(anise) failed to deploy high precision model: {}", e);
                    return Err(Error::PlanetaryData(e));
                },
            }
        } else {
            match almanac.frame_from_uid(IAU_EARTH_FRAME) {
                Ok(itrf93) => {
                    info!("(anise) deployed with low precision model");
                    itrf93
                },
                Err(e) => {
                    error!("(anise) failed to deploy low precision model: {}", e);
                    return Err(Error::PlanetaryData(e));
                },
            }
        };

        Ok(Self {
            earth_cef,
            almanac,
            files: Default::default(),
            blob: Default::default(),
        })
    }

    // Case of first deployment ever
    fn first_almanac_ever(local_storage: &str, jpl_update: bool) -> Result<Almanac, Error> {
        let mut meta = if jpl_update {
            Self::high_prec_meta()
        } else {
            Self::low_prec_meta()
        };

        let almanac = meta.process(true)?;

        if jpl_update {
            info!("(anise) high precision almanac initiated");
        } else {
            info!("(anise) low precision almanac initiated");
        }

        // setup backup storage for faster future deployments
        match meta.dumps() {
            Ok(serialized) => {
                // backup storage for faster future deployments
                let mut fd = File::create(&local_storage)
                    .unwrap_or_else(|e| panic!("almanac storage setup error: {}", e));

                fd.write_all(serialized.as_bytes())
                    .unwrap_or_else(|e| panic!("almanac storage setup error: {}", e));
            },
            Err(e) => {
                error!("(anise) serialization error: {}", e);
                warn!("(anise) local storage not initiated");
            },
        }

        Ok(almanac)
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
    #[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
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
    #[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
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
    #[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
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
    #[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
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
        // TODO: improve: only PR
        if let Some(obs) = self.observation() {
            obs.carrier().count() > 1
        } else {
            false
        }
    }
    /// [Self] cannot be True if self is compatible with PPP positioning,
    /// see <https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html#variant.PPP>
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
