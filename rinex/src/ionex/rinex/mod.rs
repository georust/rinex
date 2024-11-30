#[cfg(feature = "ionex")]
#[cfg_attr(docsrs, doc(cfg(feature = "ionex")))]
mod feature;

use crate::{
    prelude::{Rinex, RinexType},
    ionex::{IonexKey, IonexMapCoordinates, TEC},
};

use std::collections::btree_map::{Keys, Iter, IterMut};

impl Rinex {

    /// Returns true if this is a IONEX (special) [Rinex]
    pub fn is_ionex(&self) -> bool {
        self.header.rinex_type == RinexType::IonosphereMaps
    }

    /// Returns true if this IONEX only contains a single isosurface
    /// at fixed altitude.
    pub fn is_ionex_2d(&self) -> bool {
        if let Some(ionex) = &self.header.ionex {
            ionex.map_dimension == 2 && self.is_ionex()
        } else {
            false
        }
    }

    /// Returns true if this IONEX contains several isosurface spanning [Self::ionex_altitude_range].
    pub fn is_ionex_3d(&self) -> bool {
        if let Some(ionex) = &self.header.ionex {
            ionex.map_dimension == 3 && self.is_ionex()
        } else {
            false
        }
    }

    /// ```
    /// example
    /// ```
    pub fn ionex_tec_maps_keys(&self) -> Keys<'_, IonexKey, TEC> {
        if let Some(rec) = self.record.as_ionex() {
            rec.keys()
        } else {
            panic!("invalid operation");
        }
    }

    /// IONEX Total Electron Content Iterator.
    /// 
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_gzip_file("../test_resources/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    /// 
    /// for (key, tec) in rinex.ionex_tec_maps_iter() {
    ///     let latitude_ddeg = key.latitude_ddeg();
    ///     let longitude_ddeg = key.longitude_ddeg();
    ///     let altitude_km = key.altitude_km();
    ///     let tec = tec.value; // in TEC unit
    ///     if let Some(rms) = tec.rms {
    ///         // sometimes the RMS TEC is provided as well
    ///     }
    /// }
    /// ```
    pub fn ionex_tec_maps_iter(&self) -> Box<dyn Iterator<Item = (IonexKey, &TEC)> + '_> {
        if let Some(rec) = self.record.as_ionex() {
            Box::new(rec.iter().map(|(k, v)| (*k, v)))
        } else {
            Box::new([].into_iter())
        }
    }

    /// IONEX mutable TEC iterator. See [Self::ionex_tec_maps_iter()] for more information.
    pub fn ionex_tec_maps_iter_mut(&mut self) -> Box<dyn Iterator<Item = (IonexKey, &mut TEC)> + '_> {
        if let Some(rec) = self.record.as_mut_ionex() {
            Box::new(rec.iter_mut().map(|(k, v)| (*k, v)))
        } else {
            Box::new([].into_iter())
        }
    }

}
