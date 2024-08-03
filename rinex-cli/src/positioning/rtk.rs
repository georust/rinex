use crate::cli::Context;
use gnss_rtk::prelude::{BaseStation as RTKBaseStation, Carrier, Epoch, Observation, SV};
use rinex::{
    observation::ObservationData,
    prelude::{EpochFlag, Observable},
};
use std::collections::{BTreeMap, HashMap};

pub struct BaseStation<'a> {
    iter: Box<
        dyn Iterator<
                Item = (
                    &'a (Epoch, EpochFlag),
                    &'a (
                        Option<f64>,
                        BTreeMap<SV, HashMap<Observable, ObservationData>>,
                    ),
                ),
            > + 'a,
    >,
}

impl<'a> BaseStation<'a> {
    pub fn from_ctx(ctx: &'a Context) -> Self {
        let obs_rinex = ctx
            .station_data
            .as_ref()
            .unwrap_or_else(|| panic!("`rtk` requires Remote Base Station definition"))
            .observation()
            .unwrap_or_else(|| panic!("`rtk` requires OBS_RINEX from Remote Base Station"));
        Self {
            iter: obs_rinex.observation(),
        }
    }
}

impl RTKBaseStation for BaseStation<'_> {
    fn observe(&mut self, t: Epoch, sv: SV, signal: Carrier) -> Option<Observation> {
        None
    }
}
