use crate::{cli::Context, positioning::cast_rtk_carrier};

use gnss_rtk::prelude::{Epoch, Observation as RTKObservation, SV};
use rinex::{
    observation::ObservationData,
    prelude::{Carrier, EpochFlag, Observable},
};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BufKey {
    pub t: Epoch,
    pub sv: SV,
    pub carrier: Carrier,
}

pub struct RemoteRTKReference<'a> {
    buffer: HashMap<BufKey, f64>,
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
                    buffer: HashMap::with_capacity(16),
                }
            } else {
                warn!("Not RTK compatible: missing remote observations");
                Self {
                    iter: Box::new([].into_iter()),
                    buffer: HashMap::with_capacity(16),
                }
            }
        } else {
            warn!("Not RTK compatible: no reference site definition");
            Self {
                iter: Box::new([].into_iter()),
                buffer: HashMap::with_capacity(16),
            }
        }
    }
    
    pub fn observe(&mut self, t: Epoch, sv: SV, carrier: Carrier) -> Option<RTKObservation> {
        let rtk_carrier = cast_rtk_carrier(carrier);
        let mut ret = Option::<RTKObservation>::None;
        let key = BufKey { t, sv, carrier };
        if let Some(value) = self.buffer.get(&key) {
            // TODO (SNR)
            ret = Some(RTKObservation::pseudo_range(rtk_carrier, *value, None));
        }
        if ret.is_none() {
            while let Some(((remote_t, remote_flag), (_, remote_svnn))) = self.iter.next() {
                if remote_flag.is_ok() {
                    for (remote_sv, remote_obsnn) in remote_svnn.iter() {
                        for (remote_observable, remote_obs) in remote_obsnn.iter() {
                            if let Ok(remote_carrier) =
                                Carrier::from_observable(remote_sv.constellation, remote_observable)
                            {
                                let key = BufKey {
                                    t: *remote_t,
                                    sv: *remote_sv,
                                    carrier: remote_carrier,
                                };
                                let remote_rtk_carrier = cast_rtk_carrier(remote_carrier);
                                if *remote_t == t && *remote_sv == sv && remote_carrier == carrier {
                                    // TODO (SNR)
                                    return Some(RTKObservation::pseudo_range(
                                        remote_rtk_carrier,
                                        remote_obs.obs,
                                        None,
                                    ));
                                } else {
                                    self.buffer.insert(key, remote_obs.obs);
                                }
                            }
                        }
                    }
                }
            }
        }
        self.buffer.retain(|k, _| k.t >= t);
        ret
    }
}
