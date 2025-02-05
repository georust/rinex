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

    /// [KbModel] [Iterator]
    pub fn nav_klobuchar_models_iter(&self) -> Box<dyn Iterator<Item = (&NavKey, &KbModel)> + '_> {
        Box::new(
            self.nav_ionosphere_models_iter()
                .filter_map(|(k, v)| match v {
                    IonosphereModel::Klobuchar(model) => Some((k, model)),
                    _ => None,
                }),
        )
    }

    /// [BdModel] [Iterator]
    pub fn nav_bdgim_models_iter(&self) -> Box<dyn Iterator<Item = (&NavKey, &BdModel)> + '_> {
        Box::new(
            self.nav_ionosphere_models_iter()
                .filter_map(|(k, v)| match v {
                    IonosphereModel::Bdgim(model) => Some((k, model)),
                    _ => None,
                }),
        )
    }

    /// [NgModel] [Iterator]
    pub fn nav_nequickg_models_iter(&self) -> Box<dyn Iterator<Item = (&NavKey, &NgModel)> + '_> {
        Box::new(
            self.nav_ionosphere_models_iter()
                .filter_map(|(k, v)| match v {
                    IonosphereModel::NequickG(model) => Some((k, model)),
                    _ => None,
                }),
        )
    }

    // /// Returns [`KbModel`] Iterator.
    // /// RINEX4 is the real application of this, as it provides model updates
    // /// during the day. You're probably more interested
    // /// in using [ionod_correction] instead of this, especially in PPP:
    // /// ```
    // /// use rinex::prelude::*;
    // /// use rinex::navigation::KbRegionCode;
    // /// let rinex = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    // ///     .unwrap();
    // /// for (epoch, _sv, kb_model) in rinex.klobuchar_models() {
    // ///     let alpha = kb_model.alpha;
    // ///     let beta = kb_model.beta;
    // ///     assert_eq!(kb_model.region, KbRegionCode::WideArea);
    // /// }
    // /// ```
    // /// We support all RINEX3 constellations. When working with this revision,
    // /// you only get one model per day (24 hour validity period). [ionod_correction]
    // /// does that verification internally.
    // /// ```
    // /// use std::str::FromStr;
    // /// use rinex::prelude::*;
    // /// let rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
    // ///     .unwrap();
    // /// let t0 = Epoch::from_str("2021-01-01T00:00:00 UTC")
    // ///     .unwrap(); // model publication Epoch
    // /// for (t, sv, model) in rinex.klobuchar_models() {
    // ///     assert_eq!(t, t0);
    // ///     // You should use "t==t0" to compare and verify model validity
    // ///     // withint a 24 hour time frame.
    // ///     // Note that we support all RINEX3 constellations
    // ///     if sv.constellation == Constellation::BeiDou {
    // ///         assert_eq!(model.alpha.0, 1.1176E-8);
    // ///     }
    // /// }
    // /// ```
    // /// Klobuchar models exists in RINEX2 and this method applies similarly.
    // pub fn klobuchar_models(&self) -> Box<dyn Iterator<Item = (Epoch, SV, KbModel)> + '_> {
    //     Box::new(
    //         self.ionod_correction_models()
    //             .filter_map(|(t, (_, sv, ion))| ion.as_klobuchar().map(|model| (t, sv, *model))),
    //     )
    // }
    // /// Returns [`NgModel`] Iterator.
    // /// RINEX4 is the real application of this, as it provides model updates
    // /// during the day. You're probably more interested
    // /// in using [ionod_correction] instead of this, especially in PPP:
    // /// ```
    // /// use rinex::prelude::*;
    // /// let rinex = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    // ///     .unwrap();
    // /// for (epoch, ng_model) in rinex.nequick_g_models() {
    // ///     let (a0, a1, a2) = ng_model.a;
    // ///     let region = ng_model.region; // bitflag: supports bitmasking operations
    // /// }
    // /// ```
    // /// We support all RINEX3 constellations. When working with this revision,
    // /// you only get one model per day (24 hour validity period). You should prefer
    // /// [ionod_correction] which does that check internally:
    // /// ```
    // /// use std::str::FromStr;
    // /// use rinex::prelude::*;
    // /// let rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
    // ///     .unwrap();
    // /// let t0 = Epoch::from_str("2021-01-01T00:00:00 UTC")
    // ///     .unwrap(); // model publication Epoch
    // /// for (t, model) in rinex.nequick_g_models() {
    // ///     assert_eq!(t, t0);
    // ///     // You should use "t==t0" to compare and verify model validity
    // ///     // within a 24 hour time frame.
    // ///     assert_eq!(model.a.0, 66.25_f64);
    // /// }
    // /// ```
    // /// Nequick-G model is not known to RINEX2 and only applies to RINEX V>2.
    // pub fn nequick_g_models(&self) -> Box<dyn Iterator<Item = (Epoch, NgModel)> + '_> {
    //     Box::new(
    //         self.ionod_correction_models()
    //             .filter_map(|(t, (_, _, ion))| ion.as_nequick_g().map(|model| (t, *model))),
    //     )
    // }
    // /// Returns [`BdModel`] Iterator.
    // /// RINEX4 is the real application of this, as it provides model updates
    // /// during the day. You're probably more interested
    // /// in using [ionod_correction] instead of this, especially in PPP:
    // /// ```
    // /// use rinex::prelude::*;
    // /// let rinex = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    // ///     .unwrap();
    // /// for (epoch, bd_model) in rinex.bdgim_models() {
    // ///     let alpha_tecu = bd_model.alpha;
    // /// }
    // /// ```
    // /// BDGIM was introduced in RINEX4, therefore this method does not apply
    // /// to older revisions and returns an empty Iterator.
    // pub fn bdgim_models(&self) -> Box<dyn Iterator<Item = (Epoch, BdModel)> + '_> {
    //     Box::new(
    //         self.ionod_correction_models()
    //             .filter_map(|(t, (_, _, ion))| ion.as_bdgim().map(|model| (t, *model))),
    //     )
    // }

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
