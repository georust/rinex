use crate::cli::Context;
use gnss_rtk::prelude::{Carrier, Epoch, Observation, SV};
use rinex::{
    observation::ObservationData,
    prelude::{EpochFlag, Observable},
};
use std::collections::{BTreeMap, HashMap};

pub struct RemoteRTKReference<'a> {
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

impl<'a> RemoteRTKReference<'a> {
    pub fn from_ctx(ctx: &'a Context) -> Self {
        if let Some(reference_site) = ctx.reference_site.as_ref() {
            if let Some(remote_obs) = reference_site.data.observation() {
                info!("Remote reference site context created");
                Self {
                    iter: remote_obs.observation(),
                }
            } else {
                warn!("Not RTK compatible: missing remote observations");
                Self {
                    iter: Box::new([].into_iter()),
                }
            }
        } else {
            warn!("Not RTK compatible: no reference site definition");
            Self {
                iter: Box::new([].into_iter()),
            }
        }
    }
}
