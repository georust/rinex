use super::QcOpts;
//averager::Averager;
use crate::{prelude::*, *};
use horrorshow::RenderBox;
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap};

/// Observ1.0tion RINEX specific QC report
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QcReport {
    first_epoch: Epoch,
    pub has_doppler: bool,
    pub total_sv: usize,
    pub total_epochs: usize,
    pub epochs_with_obs: usize,
    pub sv_with_obs: Vec<Sv>,
    pub sv_without_obs: Vec<Sv>,
    pub total_clk: usize,
    pub anomalies: Vec<(Epoch, String)>,
    pub power_failures: Vec<(Epoch, Epoch)>,
    pub apc_estimate: (u32, (f64, f64, f64)), //nb of estimates + (ECEF)
    pub mean_ssi: HashMap<String, Vec<(Epoch, f64)>>,
    pub dcbs: HashMap<String, HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>,
    pub mp: HashMap<String, HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>,
    pub sv_angles: Option<HashMap<Sv, BTreeMap<Epoch, (f64, f64)>>>,
    pub clk_drift: HashMap<Epoch, f64>,
}

impl QcReport {
    pub fn new(rnx: &Rinex, nav: &Option<Rinex>, opts: QcOpts) -> Self {
        let mut first_epoch = Epoch::default();
        let record = rnx.record.as_obs().unwrap();
        let sv_list = rnx.space_vehicules();
        let total_sv = sv_list.len();
        let total_epochs = record.len();

        let mut has_doppler = false;
        let mut epochs_with_obs: usize = 0;
        let mut sv_with_obs: Vec<Sv> = Vec::new();
        let mut total_clk: usize = 0;

        let mut rcvr_failure: Option<Epoch> = None;
        let mut power_failures: Vec<(Epoch, Epoch)> = Vec::new();

        // RX clock
        //let mut clk_avg = Averager::new(opts.clk_drift_window);
        let mut clk_drift: HashMap<Epoch, f64> = HashMap::with_capacity(total_epochs);

        // APC
        let mut apc = (0_32, (0.0_f64, 0.0_f64, 0.0_f64));
        // SSi
        //let mut ssi_avg: HashMap<String, Averager> = HashMap::with_capacity(total_sv);
        let mut mean_ssi: HashMap<String, Vec<(Epoch, f64)>> = HashMap::with_capacity(total_sv);
        // DCBs
        // MPx
        let mut mp = rnx.observation_code_multipath();

        for (index, ((epoch, flag), (clk_offset, vehicles))) in record.iter().enumerate() {
            if index == 0 {
                first_epoch = *epoch;
            }

            if *flag == EpochFlag::PowerFailure {
                if rcvr_failure.is_none() {
                    rcvr_failure = Some(*epoch);
                }
            } else {
                // RCVR power good
                if let Some(e) = rcvr_failure {
                    power_failures.push((e, *epoch));
                }
            }

            if let Some(clk_offset) = clk_offset {
                total_clk += 1;
                /*
                 * Clock drift analysis
                if let Some(clk_avg) = clk_avg.moving_average((*clk_offset, *epoch)) {
                    clk_drift.insert(*epoch, clk_avg);
                }
                */
            }

            let mut has_obs = false;
            for (sv, observations) in vehicles {
                has_obs = observations.len() > 0;
                for (observable, data) in observations {
                    has_doppler |= observable.is_doppler_observable();
                    let carrier = "L".to_owned() + &observable.code().unwrap();
                    if !sv_with_obs.contains(&sv) {
                        sv_with_obs.push(*sv);
                    }
                    /*
                     * SSI moving average
                    if observable.is_ssi_observable() {
                        if let Some(averager) = ssi_avg.get_mut(&carrier.to_string()) {
                            if let Some(avg) = averager.moving_average((data.obs, *epoch)) {
                                if let Some(mean_ssi) = mean_ssi.get_mut(&carrier.to_string()) {
                                    mean_ssi.push((*epoch, avg));
                                } else {
                                    mean_ssi.insert(carrier.to_string(), vec![(*epoch, avg)]);
                                }
                            }
                        } else {
                            let mut avg = Averager::new(opts.obs_avg_window);
                            ssi_avg.insert(carrier.to_string(), avg);
                        }
                    }
                     */
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

        sv_with_obs.sort();

        Self {
            first_epoch,
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
            power_failures,
            anomalies: rnx.observation_epoch_anomalies(),
            total_clk,
            clk_drift,
            mean_ssi,
            dcbs: HashMap::new(),
            mp,
            sv_angles,
            apc_estimate: apc,
        }
    }
    /*
     * Window averaged data reporting
     */
    fn moving_avg_report(
        title: String,
        unit: Option<String>,
        first_epoch: Epoch,
        data: &HashMap<Epoch, f64>,
    ) -> Box<dyn RenderBox + '_> {
        box_html! {
            @ if data.len() == 0 {
                tr {
                    th {
                        b {
                            : title
                        }
                    }
                    td {
                        td {
                            : "Data missing"
                        }
                    }
                }
            } else {
                br {
                    tr {
                        th {
                            : title
                        }
                    }
                    tr {
                        th {
                            : "Epoch(k)"
                        }
                        td {
                            : first_epoch.to_string()
                        }
                        @ for (epoch, _) in data {
                            td {
                                : epoch.to_string()
                            }
                        }
                    }
                    tr {
                        th {
                            : "Epoch(k+1)"
                        }
                        @ for (epoch, _) in data.iter() {
                            td {
                                : epoch.to_string()
                            }
                        }
                    }
                    tr {
                        td {
                            b {
                                : "RMS"
                            }
                        }
                        @ for (_, value) in data {
                            @ if let Some(ref unit) = unit {
                                td {
                                    : format!("{:.3} {}", value, unit)
                                }
                            } else {
                                td {
                                    : format!("{:.3}", value)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    /*
     * Window averaged data reporting
     */
    fn code_moving_avg_report(
        title: String,
        unit: Option<String>,
        first_epoch: Epoch,
        data: &HashMap<String, Vec<(Epoch, f64)>>,
    ) -> Box<dyn RenderBox + '_> {
        box_html! {
            @ if data.len() == 0 {
                tr {
                    th {
                        b {
                            : title
                        }
                    }
                    td {
                        td {
                            : "Data missing"
                        }
                    }
                }
            } else {
                @ for signal in data.keys().sorted() {
                    br {
                        tr {
                            th {
                                : format!("{}({})", title, signal)
                            }
                        }
                        tr {
                            th {
                                : "Epoch(k)"
                            }
                            td {
                                : first_epoch.to_string()
                            }
                            @ for (epoch, _) in data[signal].iter() {
                                td {
                                    : epoch.to_string()
                                }
                            }
                        }
                        tr {
                            th {
                                : "Epoch(k+1)"
                            }
                            @ for (epoch, _) in data[signal].iter() {
                                td {
                                    : epoch.to_string()
                                }
                            }
                        }
                        tr {
                            td {
                                b {
                                    : "RMS"
                                }
                            }
                            @ for (_, value) in &data[signal] {
                                @ if let Some(ref unit) = unit {
                                    td {
                                        : format!("{:.3} {}", value, unit)
                                    }
                                } else {
                                    td {
                                        : format!("{:.3}", value)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            table {
                tr {
                    th {
                        : "Anomalies"
                    }
                    @ if self.anomalies.len() == 0 {
                        td {
                            : "None"
                        }
                    } else {
                        @ for epoch in &self.anomalies {
                            td {
                                : epoch.to_string()
                            }
                        }
                    }
                }
                tr {
                    th {
                        : "Power Failures"
                    }
                    @ if self.power_failures.len() == 0 {
                        td {
                            : "None"
                        }
                    } else {
                        @ for (start, end) in &self.power_failures {
                            td {
                                : format!("{}->{}", start, end)
                            }
                        }
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
                }
                tr {
                    td {
                        b {
                            : "Total :"
                        }
                        : self.total_epochs.to_string()
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
                }
                tr {
                    th {
                        : "PRN w/ Observation: "
                    }
                    td {
                        : pretty_sv(&self.sv_with_obs)
                    }
                }
                tr {
                    td {
                        b {
                            : "Total :"
                        }
                        : self.total_sv.to_string()
                    }
                    td {
                        : format!("{}%", self.sv_with_obs.len() * 100 / self.total_sv)
                    }
                    td {
                        : format!("{}%", self.sv_without_obs.len() * 100 / self.total_sv)
                    }
                }
                table(id="clk-drift") {
                    : Self::moving_avg_report("RX Clock Drift".to_string(), Some("s.s⁻¹".to_string()), self.first_epoch, &self.clk_drift)
                }
                table(id="ssi-analysis") {
                    : Self::code_moving_avg_report("SSI".to_string(), Some("dB".to_string()), self.first_epoch, &self.mean_ssi)
                }
                /*
                table(id="dcbs-analysis") {
                    : Self::moving_avg_report("DCBs".to_string(), self.first_epoch, &self.dcbs)
                }
                table(id="dcbs-analysis") {
                    : Self::moving_avg_report("MP".to_string(), self.first_epoch, &self.dcbs)
                }
                */
            }
        }
    }
}
