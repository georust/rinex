use horrorshow::RenderBox;
use std::collections::{HashMap, BTreeMap};
use crate::{*, prelude::*, observation::*};

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
    pub apc_estimate: (u32, (f64,f64,f64)), //nb of estimates + (ECEF)
    pub mean_ssi: HashMap<Sv, f64>,
    pub dcbs: HashMap<String, HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>,
    pub mp: HashMap<String, HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>,
    pub sv_angles: HashMap<Sv, BTreeMap<Epoch, (f64, f64)>>,
}

impl QcReport {
    pub fn new(rnx: &Rinex, nav: &Option<Rinex>) -> Self {
        let mut has_doppler = false;
        let mut epochs_with_obs: usize = 0;
        let mut sv_with_obs: Vec<Sv> = Vec::new();
        let mut total_clk: usize = 0;

        let mut rcvr_failure: Option<Epoch> = None; 
        let mut rcvr_resets: Vec<(Epoch, Epoch)> = Vec::new();

        // SSi
        let mut mean_ssi: HashMap<Sv, (u32,f64)> = HashMap::new();
        // DCBs
        let mut dcbs = rnx.observation_phase_dcb();
        // MPx
        let mut mp = rnx.observation_code_multipath();
        // APC
        let mut apc = (0_32, (0.0_f64, 0.0_f64, 0.0_f64));

        let record = rnx.record.as_obs()
            .unwrap();
        
        /*
         * Observation study
         */
        let sv_list = rnx.space_vehicules();
        let total_sv = sv_list.len();
        let total_epochs = record.len();

        for ((epoch, flag), (clk_offset, vehicles)) in record {
            if *flag == EpochFlag::PowerFailure {
                if rcvr_failure.is_none() {
                    rcvr_failure = Some(*epoch);
                }
            } else { // RCVR power good
                if let Some(e) = rcvr_failure {
                    rcvr_resets.push((e, *epoch));
                }
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

        /*
         * Augmented study
         */
        let sv_angles: Option<HashMap<Sv, BTreeMap<Epoch, (f64, f64)>>> = match nav {
            Some(nav) => {
                if let Some(ref_pos) = rnx.header.coords {
                    Some(nav.navigation_sat_angles(Some(ref_pos)))
                } else if let Some(ref_pos) = nav.header.coords {
                    Some(nav.navigation_sat_angles(Some(ref_pos)))
                } else {
                    None
                }
            },
            None => None,
        };
        
        Self {
            has_doppler,
            total_sv,
            total_epochs,
            epochs_with_obs,
            sv_without_obs: {
                sv_list
                    .iter()
                    .filter_map(|sv| {
                        if sv_with_obs.contains(&sv) {
                            Some(*sv)
                        } else {
                            None
                        }
                    })
                    .collect()
            },
            sv_with_obs,
            rcvr_resets,
            total_clk,
            mean_ssi: {
                mean_ssi.iter()
                    .map(|(sv, (n, ssi))| {
                        (*sv, ssi / *n as f64)
                    })
                    .collect()
            },
            dcbs,
            mp,
            sv_angles: sv_angles.unwrap(),
            apc_estimate: apc,
        }
    }

    pub fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            h5(id="Observations") {
                : "Observations"
            }
            table {
                tr {
                    th {
                        : "Receiver anomalies"
                    }
                }
                tr {
                    td {
                        : ""
                    }
                }
                tr {
                    th {
                        : "Epochs"
                    }
                    th {
                        : "# w/ Observation"
                    }
                    th {
                        : "# w/o Observation"
                    }
                    th {
                        : "# Total"
                    }
                }
                tr {
                    td {
                        : ""  
                    }
                    td {
                        : format!("{} [{}%]",
                            self.epochs_with_obs,
                            self.epochs_with_obs * 100 / self.total_epochs)
                    }
                    td {
                        : format!("{} [{}%]",
                            self.total_epochs - self.epochs_with_obs,
                            (self.total_epochs - self.epochs_with_obs) * 100 / self.total_epochs)
                    }
                    td {
                        : self.total_epochs.to_string()
                    }
                }
                tr {
                    th {
                        : "Sv"
                    }
                    th {
                        : "# w/ Observation"
                    }
                    th {
                        : "# w/o Observation" 
                    }
                    th {
                        : "# Total"
                    }
                }
                tr {
                    th {
                        : "Has Doppler Observations: "
                    }
                    td {
                        : self.has_doppler.to_string()
                    }
                }
                tr {
                    td {
                        : format!("{:?} [{}%]", 
                            self.sv_with_obs,
                            self.sv_with_obs.len() * 100 / self.total_sv)
                    }
                    td {
                        : format!("{:?} [{}%]", 
                            self.sv_without_obs,
                            self.sv_without_obs.len() * 100 / self.total_sv)
                    }
                    td {
                        : self.total_sv.to_string() 
                    }
                }
                //TODO
                // sv with nav
                //  Unhealthy ?
                //  if sv_angles {
                //    Rise Fall time
                //  }
                //  Obs Masked out "Possible Obs > $mask deg
                //  Deleted Obs < $mask deg
                //  Rx Clock Drift
                tr {
                    th {
                        : "Rx Clock Drift"
                    }
                }

                //  SSI
                tr {
                    th {
                        : "SSI [dB]"
                    }
                    th {
                        : "Epoch_0 - Epoch_1"
                    }
                }
                tr {
                    td {
                        : ""
                    }
                    td {
                        : "1.0"
                    }
                }
                tr {
                    th {
                        : "DCBs"
                    }
                }
                tr {
                    td {
                        : "Codes"
                    }
                    td {
                        : "RMS{Epoch(0):Epoch(1)}"
                    }
                    td {
                        : "RMS{Epoch(1):Epoch(2)}"
                    }
                }
                tr {
                    td {
                        : "10.0"
                    }
                    td {
                        : "20.0"
                    }
                }
            }
        }
    }
}
