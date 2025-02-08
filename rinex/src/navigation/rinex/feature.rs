use crate::{
    navigation::{BdModel, Ephemeris, IonosphereModel, KbModel, NavKey, NgModel},
    prelude::{
        nav::{Almanac, AzElRange, Orbit},
        Epoch, Rinex, SV,
    },
};

impl Rinex {
    /// [SV] orbital state vector determination attempt, that only applies
    /// to Navigation [Rinex].
    /// ## Inputs
    /// - sv: desired [SV]
    /// - t: desired [Epoch] to express the [Orbit]al state
    /// ## Returns
    /// - orbital state: expressed as ECEF [Orbit]
    pub fn sv_orbit(&self, sv: SV, t: Epoch) -> Option<Orbit> {
        let (toc, _, eph) = self.nav_ephemeris_selection(sv, t)?;
        eph.kepler2position(sv, toc, t)
    }

    /// [SV] (azimuth, elevation, slant range) triplet determination,
    /// that only applies to Navigation [Rinex].
    /// ## Inputs
    /// - sv: target [SV]
    /// - t: target [Epoch]
    /// - rx_orbit: RX position expressed as an [Orbit]
    /// - almanac: [Almanac] context
    /// ## Returns
    /// - [AzElRange] on calculations success
    pub fn nav_azimuth_elevation_range(
        &self,
        sv: SV,
        t: Epoch,
        rx_orbit: Orbit,
        almanac: &Almanac,
    ) -> Option<AzElRange> {
        let sv_orbit = self.sv_orbit(sv, t)?;
        let azelrange = almanac
            .azimuth_elevation_range_sez(sv_orbit, rx_orbit, None, None)
            .ok()?;
        Some(azelrange)
    }

    /// Ephemeris selection, that only applies to Navigation [Rinex].
    /// ## Inputs
    /// - sv: desired [SV]
    /// - t: target [Epoch]
    /// ## Returns
    /// - (toc, toe, [Ephemeris]) triplet if an [Ephemeris] message
    /// was decoded in the correct time frame.
    /// Note that `ToE` does not exist for GEO/SBAS [SV], so `ToC` is simply
    /// copied in this case, to maintain the API.
    pub fn nav_ephemeris_selection(&self, sv: SV, t: Epoch) -> Option<(Epoch, Epoch, &Ephemeris)> {
        let sv_ts = sv.constellation.timescale()?;

        if sv.constellation.is_sbas() {
            self.nav_ephemeris_frames_iter()
                .filter_map(|(k, eph)| {
                    if k.sv == sv {
                        Some((k.epoch, k.epoch, eph))
                    } else {
                        None
                    }
                })
                .min_by_key(|(toc, _, _)| t - *toc)
        } else {
            self.nav_ephemeris_frames_iter()
                .filter_map(|(k, eph)| {
                    if k.sv == sv && t >= k.epoch {
                        let toe = eph.toe(sv_ts)?;
                        if eph.is_valid(sv, t, toe) {
                            Some((k.epoch, toe, eph))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .min_by_key(|(toc, _, _)| t - *toc)
        }
    }

    /// Klobuchar [KbModel] Ionosphere model [Iterator].
    /// RINEX V4 is the true application of this, as it provides
    /// regular model updates (reflecting radio message stream).
    /// ```
    /// use rinex::prelude::Rinex;
    /// use rinex::navigation::KbRegionCode;
    /// let rinex = Rinex::from_gzip_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz").unwrap();
    /// for (k, model) in rinex.nav_klobuchar_models_iter() {
    ///     let (alpha, beta) = (model.alpha, model.beta);
    ///     assert_eq!(model.region, KbRegionCode::Worldwide);
    /// }
    /// ```
    ///
    /// Older RINEX revisions only provide one model per 24h time frame.
    /// This means one model per RINEX for standardized publications.
    /// ```
    /// use std::str::FromStr;
    /// use rinex::prelude::Rinex;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx").unwrap();
    /// let t0 = Epoch::from_str("2021-01-01T00:00:00 UTC").unwrap();
    /// for (k, model) in rinex.nav_klobuchar_models_iter() {
    ///     assert_eq!(k.epoch, t0); // single model
    ///     // Note that we support all RINEX3 constellations
    ///     if k.sv.constellation == Constellation::BeiDou {
    ///         assert_eq!(model.alpha.0, 1.1176E-8);
    ///     }
    /// }
    /// ```
    ///
    /// Klobuchar Ionosphere models exist in RINEX2 and this
    /// method applies similarly to the previous example.
    pub fn nav_klobuchar_models_iter(&self) -> Box<dyn Iterator<Item = (&NavKey, &KbModel)> + '_> {
        Box::new(
            self.nav_ionosphere_models_iter()
                .filter_map(|(k, v)| match v {
                    IonosphereModel::Klobuchar(model) => Some((k, model)),
                    _ => None,
                }),
        )
    }

    /// BDGIM [BdModel] Ionosphere model [Iterator].
    /// Refer to [Self::nav_klobuchar_models_iter] for similar examples.
    pub fn nav_bdgim_models_iter(&self) -> Box<dyn Iterator<Item = (&NavKey, &BdModel)> + '_> {
        Box::new(
            self.nav_ionosphere_models_iter()
                .filter_map(|(k, v)| match v {
                    IonosphereModel::Bdgim(model) => Some((k, model)),
                    _ => None,
                }),
        )
    }

    /// Nequick-G [NgModel] Ionosphere model [Iterator].
    /// Refer to [Self::nav_klobuchar_models_iter] for similar examples.
    pub fn nav_nequickg_models_iter(&self) -> Box<dyn Iterator<Item = (&NavKey, &NgModel)> + '_> {
        Box::new(
            self.nav_ionosphere_models_iter()
                .filter_map(|(k, v)| match v {
                    IonosphereModel::NequickG(model) => Some((k, model)),
                    _ => None,
                }),
        )
    }

    // /// Forms a Ut1 Provider as an [DeltaTaiUt1] Iterator from [Self] which must
    // /// be a NAV V4 RINEX file with EOP messages.
    // pub fn ut1_provider(&self) -> Box<dyn Iterator<Item = DeltaTaiUt1> + '_> {
    //     Box::new(
    //         self.earth_orientation()
    //             .map(|(t, (_, _sv, eop))| DeltaTaiUt1 {
    //                 epoch: *t,
    //                 delta_tai_minus_ut1: Duration::from_seconds(eop.delta_ut1.0),
    //             }),
    //     )
    // }
}
