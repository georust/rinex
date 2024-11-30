use crate::{cli::Context, positioning::cast_rtk_carrier};

use gnss_rtk::prelude::{Epoch, Observation as RTKObservation, SV};
use rinex::prelude::{Carrier, ObsKey, SignalObservation};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BufKey {
    pub t: Epoch,
    pub sv: SV,
    pub carrier: Carrier,
}

// TODO: this does not take SNR into account
pub struct RemoteRTKReference<'a> {
    buffer: HashMap<BufKey, f64>,
    iter: Box<dyn Iterator<Item = (ObsKey, &'a SignalObservation)> + 'a>,
}

impl<'a> RemoteRTKReference<'a> {
    /// Builds new [RemoteRTKReference] from data [Context]
    pub fn from_ctx(ctx: &'a Context) -> Self {
        if let Some(reference_site) = ctx.reference_site.as_ref() {
            if let Some(remote_obs) = reference_site.data.observation_data() {
                info!("Remote reference site context created");
                Self {
                    iter: remote_obs.signal_observations_sampling_ok_iter(),
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

    /// Signal observation attempt
    /// Inputs:
    /// - t: [Epoch] of observation
    /// - sv: [SV] signal source
    /// - carrier: [Carrier] signal
    /// Returns:
    /// - [RTKObservation] if sucessful
    pub fn observe(&mut self, t: Epoch, sv: SV, carrier: Carrier) -> Option<RTKObservation> {
        let rtk_carrier = cast_rtk_carrier(carrier);
        let mut ret = Option::<RTKObservation>::None;
        let key = BufKey { t, sv, carrier };

        // if this value has been buffered already; exppose it
        if let Some(value) = self.buffer.get(&key) {
            // TODO (SNR)
            ret = Some(RTKObservation::pseudo_range(rtk_carrier, *value, None));
        } else {
            // consume remote site, save data if they do not match,
            // return on matching success

            while let Some((k, signal)) = self.iter.next() {
                // discards invalid observations, which should not exist anyway
                let remote_carrier =
                    Carrier::from_observable(signal.sv.constellation, &signal.observable);
                if remote_carrier.is_err() {
                    continue;
                }

                let remote_carrier = remote_carrier.unwrap();

                // Build unique identifier
                let key = BufKey {
                    t: k.epoch,
                    sv: signal.sv,
                    carrier,
                };

                if k.epoch == t && signal.sv == sv && remote_carrier == carrier {
                    // perfect match
                    ret = Some(RTKObservation::pseudo_range(
                        rtk_carrier,
                        signal.value,
                        None,
                    ));
                } else {
                    // save
                    self.buffer.insert(key, signal.value);
                }
            }
        }

        self.buffer.retain(|k, _| k.t >= t);
        ret
    }
}
