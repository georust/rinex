use crate::observation::Observation;
use crate::{carrier, observation::Snr, prelude::*, processing::*, Carrier};

use super::{pretty_array, QcOpts};
use std::collections::HashMap;

//#[cfg(feature = "obs")]
//#[cfg_attr(docrs, doc(cfg(feature = "obs")))]

/*
 * GNSS signal special formatting
 */
fn report_signals(list: &Vec<Carrier>) -> String {
    let mut s = String::with_capacity(3 * list.len());
    for index in 0..list.len() - 1 {
        s.push_str(&format!(
            "{} ({:.3} MHz), ",
            list[index],
            list[index].frequency_mhz()
        ));
    }
    s.push_str(&format!(
        "{} ({:.3} MHz)",
        list[list.len() - 1],
        list[list.len() - 1].frequency_mhz()
    ));
    s
}

/*
 * Epoch anomalies formatter
 */
fn report_anomalies(anomalies: &Vec<(Epoch, EpochFlag)>) -> Box<dyn RenderBox + '_> {
    if anomalies.len() == 0 {
        box_html! {
            table(class="table is-bordered") {
                th {
                    : "Anomalies"
                }
                td {
                    : "None"
                }
            }
        }
    } else {
        box_html! {
            table(class="table is-bordered") {
                thead {
                    th {
                        : "Anomalies"
                    }
                    th {
                        : "Power failure"
                    }
                    th {
                        : "Antenna movement detected"
                    }
                    th {
                        : "Kinematic"
                    }
                    th {
                        : "External event"
                    }
                    th {
                        : "Cycle Slips"
                    }
                }
                tbody {
                    @ for (epoch, _flag) in anomalies {
                        tr {
                            td {
                                : epoch.to_string()
                            }
                            /*@match flag {
                                EpochFlag::PowerFailure => {
                                    td {
                                        : "x"
                                    }
                                },
                                EpochFlag::AntennaBeingMoved => {
                                    td {
                                        : ""
                                    }
                                    td {
                                        : "x"
                                    }
                                },
                                EpochFlag::NewSiteOccupation => {
                                    td {
                                        : ""
                                    }
                                    td {
                                        : ""
                                    }
                                    td {
                                        : "x"
                                    }
                                },
                                EpochFlag::ExternalEvent => {
                                    td {
                                        : ""
                                    }
                                    td {
                                        : ""
                                    }
                                    td {
                                        : ""
                                    }
                                    td {
                                        : "x"
                                    }
                                },
                                EpochFlag::CycleSlip => {
                                    td {
                                        : ""
                                    }
                                    td {
                                        : ""
                                    }
                                    td {
                                        : ""
                                    }
                                    td {
                                        : ""
                                    }
                                    td {
                                        : "x"
                                    }
                                },
                            }*/
                        }
                    }
                }
            }
        }
    }
}

/*
 * Epoch Epoch completion,
 * defined as at least 1 Sv with PR + PH observed on both L1 and
 * "rhs" signal,
 * also SNR condition for both signals above current mask
 */
fn report_epoch_completion(
    total: usize,
    total_with_obs: usize,
    complete: &Vec<(Carrier, usize)>,
) -> Box<dyn RenderBox + '_> {
    box_html! {
        table(class="table is-bordered") {
            tbody {
                tr {
                    th {
                        : "Total Epochs"
                    }
                    td {
                        : total.to_string()
                    }
                }
                tr {
                    th {
                        : "w/ observations"
                    }
                    td {
                        : format!("{} ({}%)", total_with_obs, total_with_obs * 100 / total)
                    }
                }
                tr {
                    th {
                        : "Complete"
                    }
                    @ for (signal, count) in complete {
                        td {
                            b {
                                : format!("L1/{}", signal)
                            }
                            p {
                                : count.to_string()
                            }
                            b {
                                : format!("{}%", count * 100/total)
                            }
                            //: format!("<b>L1/{}</b>: {} (<b>{}%</b>)", signal, count, count *100/total)
                        }
                    }
                }
            }
        }
    }
}

/*
 * Reports statistical analysis results for SSx observations
 */
fn report_ssi_statistics(
    ssi_stats: &HashMap<Observable, (f64, f64, f64)>,
) -> Box<dyn RenderBox + '_> {
    box_html! {
        table(class="table is-bordered") {
            thead {
                tr {
                    td {
                        : ""
                    }
                    @ for (signal, _) in ssi_stats {
                        th {
                            : signal.to_string()
                        }
                    }
                }
            }
            tbody {
                tr {
                    th {
                        : "Mean"
                    }
                    @ for (_, (mean, _, _)) in ssi_stats {
                        td {
                            : format!("{:.3} dB", mean)
                        }
                    }
                }
                tr {
                    th {
                        : "Deviation" // (&#x03C3;)"
                    }
                    @ for (_, (_, sigma, _)) in ssi_stats {
                        td {
                            : format!("{:.3} dB", sigma)
                        }
                    }
                }
                tr {
                    th {
                        : "Skewness" //(&#x03BC; / &#x03C3;&#x03B3;)"
                    }
                    @ for (_, (_, _, sk)) in ssi_stats {
                        td {
                            : format!("{:.3}", sk)
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct QcObsAnalysis {
    /// list of observables identified
    observables: Vec<String>,
    /// list of signals identified
    signals: Vec<Carrier>,
    /// list of codes encountered
    codes: Vec<String>,
    /// true if doppler observation is present
    has_doppler: bool,
    /// Abnormal events, by chronological epochs
    anomalies: Vec<(Epoch, EpochFlag)>,
    /// Total epochs
    total_epochs: usize,
    /// Epochs with at least 1 observation
    total_with_obs: usize,
    /// Complete epochs, with respect to given signal
    complete_epochs: Vec<(Carrier, usize)>,
    /// Min. Max. SNR (sv @ epoch)
    #[cfg(feature = "obs")]
    #[cfg_attr(docrs, doc(cfg(feature = "obs")))]
    min_max_snr: ((Sv, Epoch, Snr), (Sv, Epoch, Snr)),
    /// SSi statistical analysis (mean, stddev, skew)
    #[cfg(feature = "obs")]
    #[cfg_attr(docrs, doc(cfg(feature = "obs")))]
    ssi_stats: HashMap<Observable, (f64, f64, f64)>,
    /// clock_drift
    clock_drift: Option<f64>,
}

impl QcObsAnalysis {
    pub fn new(rnx: &Rinex, _nav: &Option<Rinex>, opts: &QcOpts) -> Self {
        let sv = rnx.space_vehicles();
        let obs = rnx.header.obs.as_ref().unwrap();
        let mut observables = obs.codes.clone();
        let observables = observables.get_mut(&sv[0].constellation).unwrap();
        let mut signals: Vec<Carrier> = Vec::new();
        let mut codes: Vec<String> = Vec::new();
        let mut anomalies: Vec<(Epoch, EpochFlag)> = Vec::new();
        let mut total_epochs: usize = 0;
        let mut epoch_with_obs: Vec<Epoch> = Vec::new();
        let mut complete_epochs: HashMap<Carrier, usize> = HashMap::new();
        let mut min_max_snr = (
            (Sv::default(), Epoch::default(), Snr::DbHz54),
            (Sv::default(), Epoch::default(), Snr::DbHz0),
        );
        let mut ssi_stats: HashMap<Observable, (f64, f64, f64)> = HashMap::new();

        let clock_drift: Option<f64> = match rnx.record.as_obs() {
            Some(r) => {
                let mask: Filter = Filter::from(MaskFilter {
                    operand: MaskOperand::Equals,
                    item: TargetItem::ClockItem,
                });
                let _clk_data = r.filter(mask);
                //let der = clk_data.derivative();
                //let mov = der.moving_average(Duration::from_seconds(600.0), None);
                //TODO
                None
            },
            _ => None,
        };

        if let Some(r) = rnx.record.as_obs() {
            total_epochs = r.len();
            for ((epoch, flag), (clk, svs)) in r {
                if let Some(_clk) = clk {}

                if !flag.is_ok() {
                    anomalies.push((*epoch, *flag));
                }
                for (sv, observables) in svs {
                    if observables.len() > 0 && !epoch_with_obs.contains(&epoch) {
                        epoch_with_obs.push(*epoch);
                    }

                    for (observable, observation) in observables {
                        let code = observable.code().unwrap();
                        let carrier = observable.carrier(sv.constellation).unwrap();
                        if !signals.contains(&carrier) {
                            signals.push(carrier);
                        }
                        if !codes.contains(&code) {
                            codes.push(code);
                        }

                        if let Some(snr) = observation.snr {
                            if snr < min_max_snr.0 .2 {
                                min_max_snr.0 .0 = *sv;
                                min_max_snr.0 .1 = *epoch;
                                min_max_snr.0 .2 = snr;
                            }
                            if snr > min_max_snr.1 .2 {
                                min_max_snr.1 .0 = *sv;
                                min_max_snr.1 .1 = *epoch;
                                min_max_snr.1 .2 = snr;
                            }
                        }
                    }
                }
            }

            /*
             * Now that signals have been determined,
             * determine observation completion
             */
            for (_, (_, svs)) in r {
                let mut complete: HashMap<Carrier, bool> = HashMap::new();
                for (sv, observables) in svs {
                    for (observable, observation) in observables {
                        if !observable.is_phase_observable() {
                            if !observable.is_pseudorange_observable() {
                                continue;
                            }
                        }
                        /*
                         * SNR condition
                         */
                        if let Some(snr) = observation.snr {
                            if snr < Snr::from(opts.min_snr_db) {
                                continue; // not to be considered
                            }
                        } else {
                            if observable.is_phase_observable() {
                                continue; // phase should have SNR information attached to it
                            }
                        }

                        /*
                         * Signal condition
                         */
                        let carrier_code = &observable.to_string()[1..2];
                        if carrier_code == "1" {
                            // we only search for other signals
                            continue;
                        }

                        let _code = observable.code().unwrap();
                        let carrier = observable.carrier(sv.constellation).unwrap();
                        if let Some(complete) = complete.get_mut(&carrier) {
                            if !*complete {
                                for k_code in carrier::KNOWN_CODES.iter() {
                                    if !k_code.starts_with("1") {
                                        continue; // we're looking for a "1" reference
                                    }
                                    let to_find = match observable.is_phase_observable() {
                                        true => "C".to_owned() + k_code,  // looking for PR
                                        false => "L".to_owned() + k_code, // looking for PH
                                    };
                                    for (observable, _observation) in observables {
                                        if observable.to_string() == to_find {
                                            *complete = true;
                                        }
                                    }
                                    if *complete {
                                        break;
                                    }
                                }
                            }
                        } else {
                            complete.insert(carrier, false);
                        }
                    }
                }
                for (carrier, completed) in complete {
                    if completed {
                        if let Some(count) = complete_epochs.get_mut(&carrier) {
                            *count += 1;
                        } else {
                            complete_epochs.insert(carrier, 1);
                        }
                    }
                }
            }

            /*
             * SSI statistical analysis
             */
            let mut mean_ssi: HashMap<_, _> = r.mean_observable();
            mean_ssi.retain(|obs, _| obs.is_ssi_observable());
            for (obs, mean) in mean_ssi {
                ssi_stats.insert(obs.clone(), (mean, 0.0_f64, 0.0_f64));
            }

            let mut stddev_ssi: HashMap<_, _> = r.mean_observable(); // TODO r.stddev_observable();
            stddev_ssi.retain(|obs, _| obs.is_ssi_observable());
            for (obs, stddev) in stddev_ssi {
                if let Some((_, dev, _)) = ssi_stats.get_mut(&obs) {
                    *dev = stddev;
                }
            }

            let mut skew_ssi: HashMap<_, _> = r.mean_observable(); // TODO r.skewness_observable();
            skew_ssi.retain(|obs, _| obs.is_ssi_observable());
            for (obs, skew) in skew_ssi {
                if let Some((_, _, sk)) = ssi_stats.get_mut(&obs) {
                    *sk = skew;
                }
            }
        }

        codes.sort();
        signals.sort();
        observables.sort();

        Self {
            observables: { observables.iter().map(|v| v.to_string()).collect() },
            has_doppler: {
                let mut ret = false;
                for obs in observables.iter() {
                    if obs.is_doppler_observable() {
                        ret = true;
                        break;
                    }
                }
                ret
            },
            codes,
            signals,
            anomalies,
            total_epochs,
            total_with_obs: epoch_with_obs.len(),
            complete_epochs: {
                let mut ret: Vec<(Carrier, usize)> =
                    complete_epochs.iter().map(|(k, v)| (*k, *v)).collect();
                ret.sort();
                ret
            },
            min_max_snr,
            ssi_stats,
            clock_drift,
        }
    }
}

use crate::qc::HtmlReport;
use horrorshow::RenderBox;

impl HtmlReport for QcObsAnalysis {
    fn to_html(&self) -> String {
        todo!()
    }
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            tr {
                th {
                    : "Signals"
                }
                td {
                    : report_signals(&self.signals)
                }
            }
            tr {
                th {
                    : "Codes"
                }
                td {
                    : pretty_array(&self.codes)
                }
            }
            tr {
                th {
                    : "Observables"
                }
                td {
                    : pretty_array(&self.observables)
                }
            }
            tr {
                th {
                    : "Has Doppler"
                }
                @ if self.has_doppler {
                    td {
                        : "True"
                    }
                } else {
                    td {
                        : "False"
                    }
                }
            }
            div(class="table-container") {
                : report_anomalies(&self.anomalies)
            }
            div(class="epoch-completion") {
                : report_epoch_completion(self.total_epochs, self.total_with_obs, &self.complete_epochs)
            }
            table(class="table is-bordered") {
                thead {
                    tr {
                        th {
                            : "Worst SNR"
                        }
                        th {
                            : "Best SNR"
                        }
                    }
                }
                tbody {
                    tr {
                        td {
                            p {
                                :  self.min_max_snr.0.0.to_string()
                            }
                            b {
                                : format!("{:e}", self.min_max_snr.0.2)
                            }
                            p {
                                : format!("@{}", self.min_max_snr.0.1)
                            }
                        }
                        td {
                            p {
                                :  self.min_max_snr.1.0.to_string()
                            }
                            b {
                                : format!("{:e}", self.min_max_snr.1.2)
                            }
                            p {
                                : format!("@{}", self.min_max_snr.1.1)
                            }
                        }
                    }
                }
            }
            div(class="epoch-completion") {
                : report_ssi_statistics(&self.ssi_stats)
            }
        }
    }
}
