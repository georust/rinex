use crate::{carrier, observation::Snr, prelude::*, Carrier};
use itertools::Itertools;

use super::{pretty_array, QcOpts};
use std::collections::HashMap;

use statrs::statistics::Statistics;

#[cfg(feature = "processing")]
use crate::preprocessing::*; // include preprocessing toolkit when feasible,
                             // for complete analysis

#[cfg(feature = "obs")]
use crate::observation::Observation; // having this feature unlocks full OBS RINEX analysis

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
 * Report RX Clock drift analysis
 */
fn report_clock_drift(data: &Vec<(Epoch, f64)>) -> Box<dyn RenderBox + '_> {
    box_html! {
        @ if data.is_empty() {
            table(class="table is-bordered") {
                tr {
                    th {
                        : "Unfeasible"
                    }
                    td {
                        : "Missing Data"
                    }
                }
            }
        } else {
            table(class="table is-bordered") {
                tr {
                    th {
                        : "Epoch"
                    }
                    th {
                        : "Mean Clock drift [s/s]"
                    }
                }
                @ for (epoch, drift) in data {
                    tr {
                        td {
                            : epoch.to_string()
                        }
                        td {
                            : format!("{:e}", drift)
                        }
                    }
                }
            }
        }
    }
}

/*
 * Epoch anomalies formatter
 */
fn report_anomalies<'a>(
    cs: &'a Vec<Epoch>,
    power: &'a Vec<Epoch>,
    other: &'a Vec<(Epoch, EpochFlag)>,
) -> Box<dyn RenderBox + 'a> {
    box_html! {
                tr {
                    th {
                        : "Power Failures"
                    }
                    @ if power.is_empty() {
                        td {
                            : "None"
                        }
                    } else {
                        td {
                            : format!("{:?}", power)
                        }
                        tr {
                            th {
                                : "Longest"
                            }
                            td {
                                //: power.iter().max_by(|(_, d1), (_, d2)| d1.cmp(d2)).unwrap().to_string()
                                : "TODO"
                            }
                            td {
                                : "Average Power failure"
                            }
                            td {
                                : "TODO"
                            }
                        }
                    }
                }
                tr {
                    th {
                        : "Cycle slip(s)"
                    }
                    @ if cs.is_empty() {
                        td {
                            : "None"
                        }
                    } else {
                        td {
                            : format!("{:?}", cs)
                        }
                    }
                }
                tr {
                    th {
                        : "Other anomalies"
                    }
                    @ if other.is_empty() {
                        td {
                            : "None"
                        }
                    } else {
                        td {
                            : "Epoch"
                        }
                        td {
                            : "Event"
                        }
                        @ for (e, event) in other {
                            td {
                                : ""
                            }
                            td {
                                : e.to_string()
                            }
                            //@ match event {
                            //    EpochFlag::AntennaBeingMoved => {
                            //        td {
                            //            : "Antenna being moved"
                            //        }
                            //    },
                            //    _ => {},
                            //}
                            //        }
                            //td {
                            //    @ match event {
                            //        EpochFlag::NewSiteOccupation => {
                            //            : "New Site Occupation"
                            //        }
                            //        EpochFlag::ExternalEvent => {
                            //            : "External Event"
                            //        }
                            //        _ => {
                            //            : "Other"
                            //        }
                            //    }
                            //}
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
                tr {
                    th {
                        : "Total#"
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

/*
 * SNR analysis report
 */
fn report_snr_statistics(
    snr_stats: &HashMap<Observable, ((Epoch, f64), (Epoch, f64))>,
) -> Box<dyn RenderBox + '_> {
    box_html! {
        @ for (observable, _) in snr_stats {
            tr {
                th {
                    : observable.to_string()
                }
            }
        }
        tr {
            th {
                : "Worst"
            }
            @ for (_, (min, _)) in snr_stats {
                td {
                    : format!("{:e} @{}", min.0, min.1)
                }
            }
        }
    }
}

/*
 * Reports statistical analysis results for SSx observations
 */
fn report_ssi_statistics(ssi_stats: &HashMap<Observable, (f64, f64)>) -> Box<dyn RenderBox + '_> {
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
                    @ for (_, (mean, _)) in ssi_stats {
                        td {
                            : format!("{:.3} dB", mean)
                        }
                    }
                }
                tr {
                    th {
                        : "Deviation" // (&#x03C3;)"
                    }
                    @ for (_, (_, std)) in ssi_stats {
                        td {
                            : format!("{:.3} dB", std)
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
/// OBS RINEX specific QC analysis.  
/// Full OBS RINEX analysis requires both the "obs" and "processing" features.
pub struct QcObsAnalysis {
    /// Identified Observables
    observables: Vec<String>,
    /// Identified Signals
    signals: Vec<Carrier>,
    /// Codes that were idenfitied
    codes: Vec<String>,
    /// true if doppler observations were identified
    has_doppler: bool,
    /// CS anomalies
    cs_anomalies: Vec<Epoch>,
    /// Epochs where power failures took place, and their duration
    power_failures: Vec<Epoch>,
    /// Other abnormal events, by chronological epochs
    other_anomalies: Vec<(Epoch, EpochFlag)>,
    /// Total number of epochs identified
    total_epochs: usize,
    /// Epochs with at least 1 observation
    total_with_obs: usize,
    /// Complete epochs, with respect to given signal
    complete_epochs: Vec<(Carrier, usize)>,
    #[cfg(feature = "obs")]
    /// Min. & Max. SNR (signal @ epoch)
    snr_stats: HashMap<Observable, ((Epoch, f64), (Epoch, f64))>,
    #[cfg(feature = "obs")]
    /// SSI statistical analysis (mean, stddev)
    ssi_stats: HashMap<Observable, (f64, f64)>,
    #[cfg(feature = "obs")]
    /// RX clock drift
    clock_drift: Vec<(Epoch, f64)>,
}

impl QcObsAnalysis {
    pub fn new(rnx: &Rinex, opts: &QcOpts) -> Self {
        let doppler_obs = rnx.observable().filter(|obs| obs.is_doppler_observable());

        let mut observables: Vec<String> = rnx.observable().map(|obs| obs.to_string()).collect();

        let mut signals: Vec<_> = rnx.carrier().unique().collect();
        let mut codes: Vec<_> = rnx.code().map(|c| c.to_string()).collect();

        let cs_anomalies: Vec<_> = rnx
            .epoch_anomalies()
            .filter_map(|(e, flag)| {
                if flag == EpochFlag::CycleSlip {
                    Some(e)
                } else {
                    None
                }
            })
            .collect();

        let power_failures: Vec<_> = rnx
            .epoch_anomalies()
            .filter_map(|(e, flag)| {
                if flag == EpochFlag::PowerFailure {
                    Some(e)
                } else {
                    None
                }
            })
            .collect();

        let other_anomalies: Vec<_> = rnx
            .epoch_anomalies()
            .filter_map(|(e, flag)| {
                if flag != EpochFlag::PowerFailure && flag != EpochFlag::CycleSlip {
                    Some((e, flag))
                } else {
                    None
                }
            })
            .collect();

        let mut total_epochs = rnx.epoch().count();
        let mut epoch_with_obs: Vec<Epoch> = Vec::new();
        let mut complete_epochs: HashMap<Carrier, usize> = HashMap::new();

        if let Some(r) = rnx.record.as_obs() {
            total_epochs = r.len();
            for ((epoch, flag), (clk, svs)) in r {
                for (sv, observables) in svs {
                    if !observables.is_empty() {
                        if !epoch_with_obs.contains(&epoch) {
                            epoch_with_obs.push(*epoch);
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
        }
        // append ssi: drop vehicle differentiation
        let mut ssi: HashMap<Observable, Vec<f64>> = HashMap::new();
        for (_, _, obs, value) in rnx.ssi() {
            if let Some(values) = ssi.get_mut(&obs) {
                values.push(value);
            } else {
                ssi.insert(obs.clone(), vec![value]);
            }
        }
        /*
         * SSI statistical analysis: {mean, stddev,}
         * per signal: we do not differentiate vehicles
         */
        let ssi_stats: HashMap<Observable, (f64, f64)> = ssi
            .iter()
            .map(|(obs, values)| (obs.clone(), (values.mean(), values.std_dev())))
            .collect();
        // append snr: drop vehicle differentiation
        let mut snr: HashMap<Observable, Vec<(Epoch, f64)>> = HashMap::new();
        for (e, _, obs, value) in rnx.snr() {
            let value_f64: f64 = (value as u8).into();
            if let Some(values) = snr.get_mut(&obs) {
                values.push((e.0, value_f64));
            } else {
                snr.insert(obs.clone(), vec![(e.0, value_f64)]);
            }
        }
        /*
         * SNR analysis: {min, max}
         * per signal: we do not differentiate vehicles
         */
        let snr_stats: HashMap<Observable, ((Epoch, f64), (Epoch, f64))> = snr
            .iter()
            .map(|(obs, values)| {
                let min = Statistics::min(values.1);
                let max = Statistics::max(values.1);
                (
                    obs.clone(),
                    ((Epoch::default(), min), (Epoch::default(), max)),
                )
            })
            .collect();
        /*
         * sort, prior reporting
         */
        codes.sort();
        observables.sort();
        signals.sort();

        Self {
            codes,
            signals,
            observables,
            has_doppler: doppler_obs.count() > 0,
            cs_anomalies,
            power_failures,
            other_anomalies,
            total_epochs,
            total_with_obs: epoch_with_obs.len(),
            complete_epochs: {
                let mut ret: Vec<(Carrier, usize)> =
                    complete_epochs.iter().map(|(k, v)| (*k, *v)).collect();
                ret.sort();
                ret
            },
            snr_stats,
            ssi_stats,
            clock_drift: {
                let rx_clock: Vec<_> = rnx
                    .recvr_clock()
                    .map(|((e, flag), value)| (e, value))
                    .collect();
                let der = Derivative::new(1);
                let rx_clock_drift: Vec<(Epoch, f64)> = der.eval(rx_clock);
                let mov = Averager::mov(opts.clock_drift_window);
                mov.eval(rx_clock_drift)
            },
        }
    }
}

use crate::qc::HtmlReport;
use horrorshow::RenderBox;

impl HtmlReport for QcObsAnalysis {
    fn to_html(&self) -> String {
        unreachable!("never used by itself")
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
            tr {
                table(class="table is-bordered") {
                    thead {
                        th {
                            : "Anomalies"
                        }
                    }
                    tbody {
                        : report_anomalies(&self.cs_anomalies, &self.power_failures, &self.other_anomalies)
                    }
                }
            }
            tr {
                table(class="table is-bordered") {
                    thead {
                        th {
                            : "Epochs"
                        }
                    }
                    tbody {
                        : report_epoch_completion(self.total_epochs, self.total_with_obs, &self.complete_epochs)
                    }
                }
            }
            tr {
                table(class="table is-bordered") {
                    thead {
                        th {
                            : "SNR"
                        }
                    }
                    tbody {
                        : report_snr_statistics(&self.snr_stats)
                    }
                }
            }
            tr {
                table(class="table is-bordered") {
                    thead {
                        th {
                            : "SSI"
                        }
                    }
                    tbody {
                        : report_ssi_statistics(&self.ssi_stats)
                    }
                }
            }
            tr {
                table(class="table is-bordered") {
                    thead {
                        th {
                            : "(RX) Clock Drift"
                        }
                    }
                    tbody {
                        : report_clock_drift(&self.clock_drift)
                    }
                }
            }
        }
    }
}
