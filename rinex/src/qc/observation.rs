use crate::{prelude::*, observation::*};
use std::collections::{HashMap, BTreeMap};

/// Observation RINEX specific QC report
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QcReport {
    pub has_doppler: bool,
    pub total_sv: usize,
    pub total_epochs: usize,
    pub epochs_with_obs: usize,
    pub sv_with_obs: Vec<Sv>,
    pub sv_without_obs: Vec<Sv>,
    pub total_clk: usize,
    pub rcvr_resets: Vec<(Epoch, Epoch)>,
    pub mean_ssi: HashMap<Sv, f64>,
    pub dcbs: HashMap<String, HashMap<Sv, BTreeMap<Epoch, f64>>>,
    pub mp: HashMap<String, HashMap<Sv, BTreeMap<Epoch, f64>>>,
}

impl QcReport {
    pub fn new(rnx: &Rinex, nav: &Option<Rinex>) -> Self {
        let mut has_doppler = false;
        let mut epochs_with_obs: usize = 0;
        let mut sv_with_obs: Vec<Sv> = Vec::new();
        let mut total_clk: usize = 0;

        let mut last_reset: Option<Epoch> = None;
        let mut rcvr_resets: Vec<(Epoch,Epoch)> = Vec::new();

        let mut mean_ssi: HashMap<Sv, (u32,f64)> = HashMap::new();

        let record = rnx.record.as_obs()
            .unwrap();

        let sv_list = rnx.space_vehicules();
        let total_sv = sv_list.len();
        let total_epochs = record.len();

        for ((epoch, flag), (clk_offset, vehicles)) in record {
            if *flag == EpochFlag::PowerFailure {
                if let Some(prev) = last_reset {
                    time_between_resets.0 += 1;
                    time_between_resets.1 += *epoch - prev;
                }
                last_reset = Some(*epoch);
            }
            if clk_offset.is_some() {
                total_clk += 1;
            }
            let mut has_obs = false;
            for (sv, observations) in vehicles {
                has_obs = observations.len() > 0;
                for (code, data) in observations {
                    has_doppler |= is_doppler_obs_code!(code);
                    if is_sig_strength_obs_code!(code) {
                        if let Some((n, ssi)) = mean_ssi.get_mut(sv) {
                            *n += 1;
                            *ssi += data.obs;
                        } else {
                            mean_ssi.insert(*sv, (1, data.obs));
                        }
                    }
                }
            }
            if has_obs {
                epochs_with_obs += 1; 
            }
        }
        Self {
            has_doppler,
            epochs_with_obs,
            sv_with_obs,
            sv_without_obs: {
                sv_list.iter()
                    .filter_map(|sv| {
                        if sv_with_obs.contains(&sv) {
                            Some(sv)
                        } else {
                            None
                        }
                    })
                    .collect()
            },
            rcvr_resets,
            total_clk,
            mean_ssi: {
                mean_ssi.iter_mut()
                    .map(|(sv, (n, ssi)| {
                        sv, ssi / n as f64
                    })
                    .collect();
            },
            dcbs: HashMap::new(),
            mp: HashMap::new(),
        }
    }
}
