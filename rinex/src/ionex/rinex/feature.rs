use crate::{
    ionex::{IonexKey, TEC},
    prelude::{Epoch, Rinex},
};

impl Rinex {
    /// Returns IONEX TEC map borders.
    /// Note that this is only based on Header definitions. If provided
    /// content did not follow those specs (incorrect), the returned value here will not
    /// reflect actual content.
    /// ## Output
    /// - (lat_min, lat_max): southernmost and northernmost latitude, in decimal degrees
    /// - (long_min, long_max): easternmost and westernmost longitude, in decimal degrees
    pub fn tec_map_borders(&self) -> Option<((f64, f64), (f64, f64))> {
        let ionex = self.header.ionex.as_ref()?;
        Some((
            (ionex.grid.latitude.start, ionex.grid.latitude.end),
            (ionex.grid.longitude.start, ionex.grid.longitude.end),
        ))
    }

    /// Designs an iterator over RMS TEC exclusively
    /// ```
    /// use rinex::prelude::*;
    ///
    /// let rnx = Rinex::from_gzip_file("../test_resources/IONEX/V1/jplg0010.17i.gz")
    ///     .unwrap();
    ///
    /// for (t, lat, lon, alt, rms) in rnx.ionex_rms_tec_maps_iter() {
    ///     // t: Epoch
    ///     // lat: ddeg
    ///     // lon: ddeg
    ///     // alt: km
    ///     // rms|TECu| (f64)
    /// }
    /// ```
    pub fn ionex_rms_tec_maps_iter(&self) -> Box<dyn Iterator<Item = (IonexKey, f64)> + '_> {
        Box::new(self.ionex_tec_maps_iter().filter_map(|(k, tec)| {
            if let Some(rms) = tec.rms_tec() {
                Some((k, rms))
            } else {
                None
            }
        }))
    }

    /// Returns fixed altitude (in kilometers) if this is a 2D IONEX (only).
    /// Note that this is only based on Header definitions. If provided
    /// content did not follow those specs (incorrect), the returned value here will not
    /// reflect actual content.
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/IONEX/V1/jplg0010.17i.gz")
    ///     .unwrap();
    /// assert_eq!(rnx.ionex_fixed_altitude_km(), Some(450.0));
    ///
    /// let rnx = Rinex::from_file("../test_resources/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    /// assert_eq!(rnx.ionex_fixed_altitude_km(), Some(350.0));
    /// ```
    pub fn ionex_fixed_altitude_km(&self) -> Option<f64> {
        if self.is_ionex_2d() {
            let header = self.header.ionex.as_ref()?;
            Some(header.grid.height.start)
        } else {
            None
        }
    }

    /// Returns altitude range of this 3D IONEX TEC maps, expressed as {min, max}
    /// both in kilometers. Returns None if this is not a 3D IONEX.
    /// Note that this is only based on Header definitions. If provided
    /// content did not follow those specs (incorrect), the returned value here will not
    /// reflect actual content.
    ///
    /// ```
    /// example
    /// ```
    pub fn ionex_tec_maps_altitude_range_km(&self) -> Option<(f64, f64)> {
        if self.is_ionex_3d() {
            let header = self.header.ionex.as_ref()?;
            Some((header.grid.height.start, header.grid.height.end))
        } else {
            None
        }
    }

    /// Designs an TEC isosurface iterator starting at lowest altitude,
    /// ending at highest altitude. See [Self::ionex_tec_maps_altitude_range] for
    /// useful information.
    pub fn ionex_tec_isosurface_iter(&self) -> Box<dyn Iterator<Item = (IonexKey, TEC)> + '_> {
        Box::new([].into_iter())
    }

    /// Designs an RMS TEC isosurface iterator starting at lowest altitude,
    /// ending at highest altitude. See [Self::ionex_tec_maps_altitude_range] for
    /// useful information.
    pub fn ionex_rms_tec_isosurface_iter(&self) -> Box<dyn Iterator<Item = (IonexKey, TEC)> + '_> {
        Box::new([].into_iter())
    }
}
