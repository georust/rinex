
use anise::prelude::Orbit;
use rinex::prelude::{Rinex, TimeScale};
use crate::context::Error;

use std::path::Path;

mod rinex_ctx;

#[cfg(feature = "sp3")]
#[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
mod sp3_ctx;

pub mod meta;
use meta::MetaData;

pub mod global;

use std::collections::{
    HashMap,
    hash_map::Keys,
};

use itertools::Itertools;

#[cfg(feature = "sp3")]
use sp3::prelude::SP3;

pub enum UserData {
    RINEX(Rinex),
    #[cfg(feature = "sp3")]
    SP3(SP3),
}

#[derive(Default)]
pub struct DataSet {
    /// Ground position as ideally described by provided [UserData],
    /// expressed as an [Orbit].
    /// This is an RX position that may serve many purposes, whether
    /// this is a static or a roaming application.
    pub ground_position: Option<Orbit>,
    /// Internal [UserData] storaged, sorted by [MetaData]
    pub data: HashMap<MetaData, UserData>,
}

impl DataSet {

    /// Smart data loader, that will automatically pick up the provided
    /// format (if supported) and load it into this [DataSet].
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let path = path.as_ref();
        if let Ok(rinex) = Rinex::from_file(path) {
            self.load_rinex(path, rinex)?;
            info!(
                "RINEX: \"{}\" has been loaded",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            Ok(())
        } else if let Ok(sp3) = SP3::from_file(path) {
            self.load_sp3(path, sp3)?;
            info!(
                "SP3: \"{}\" has been loaded",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            Ok(())
        } else {
            Err(Error::NonSupportedFormat)
        }
    }

    /// Smart data loader, that will automatically pick up the provided
    /// format (if supported).
    #[cfg(feature = "flate2")]
    pub fn load_gzip_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let path = path.as_ref();
        if let Ok(rinex) = Rinex::from_gzip_file(path) {
            self.load_rinex(path, rinex)?;
            info!(
                "RINEX: \"{}\" has been loaded",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            Ok(())
        } else if let Ok(sp3) = SP3::from_gzip_file(path) {
            self.load_sp3(path, sp3)?;
            info!(
                "SP3: \"{}\" has been loaded",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            Ok(())
        } else {
            Err(Error::NonSupportedFormat)
        }
    }


    /// Define or overwrite the internal Ground position, expressed as [Orbit].
    /// Refer to its definition for more information
    pub fn set_ground_position(&mut self, rx: Orbit) {
        self.ground_position = Some(rx);
    }

    /// Returns a [TimeScale] that best fit this [DataSet].
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

    /// [MetaData] iterator for this [DataSet]
    pub fn meta_iter(&self) -> Keys<'_, MetaData, UserData> {
        self.data.keys()
    }

    /// Filename iterator for this [DataSet]
    pub fn files_iter(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(self.data.keys().map(|k| k.name.clone()).sorted())
    }

    /// Returns true when only a single file has been loaded in [QcContext]
    pub fn single_file(&self) -> bool {
        self.meta_iter()
            .map(|meta| meta.name.clone())
            .unique()
            .count()
            == 1
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

    /// Returns path to File considered as Primary input product.
    /// If [QcContext] only holds one input product, it is obviously defined as Primary,
    /// whatever its kind.
    pub fn primary_name(&self) -> Option<String> {
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
            if let Some(first) = self
                .meta_iter()
                .filter(|meta| meta.product_id == product)
                .next()
            {
                return Some(first.name.to_string());
            }
        }
        None
    }



    /// True if [DataSet] is compatible with PPP positioning,
    /// see <https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html#variant.PPP>
    pub fn is_ppp_compatible(&self) -> bool {
        // TODO: check PH as well
        self.is_cpp_compatible()
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


}