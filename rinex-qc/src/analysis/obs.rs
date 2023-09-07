use crate::{carrier, observation::Snr, prelude::*, Carrier};
use itertools::Itertools;
use std::str::FromStr;
use horrorshow::RenderBox;
use rinex_qc_traits::HtmlReport;

use super::{pretty_array, QcOpts};
use std::collections::HashMap;

use statrs::statistics::Statistics;

#[cfg(feature = "processing")]
use crate::preprocessing::*; // include preprocessing toolkit when feasible,
                             // for complete analysis

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
                        : "Average Duration"
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
                        : format!("{}", e)
                    }
                    @ if *event == EpochFlag::AntennaBeingMoved {
                        td {
                            : "Antenna Being Moved"
                        }
                    } else if *event == EpochFlag::NewSiteOccupation {
                        td {
                            : "New Site Occupation"
                        }
                    } else if *event == EpochFlag::ExternalEvent {
                        td {
                            : "External Event"
                        }
                    } else {
                        td {
                            : "Other"
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
                    td {
                        b {
                            : "Complete"
                        }
                        p {
                            : "Epochs with at least Phase + PR"
                        }
                        p {
                            : "in dual frequency, with"
                        }
                        p {
                            : "both SNR and elev above masks"
                        }
                    }
                    @ for (signal, count) in complete {
                        td {
                            b {
                                : format!("L1/{}", signal)
                            }
                            p {
                                : format!("{} ({}%)", count, count * 100 / total)
                            }
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
        tr {
            td {
                : ""
            }
            @ for (observable, _) in snr_stats {
                @ if observable.is_phase_observable() || observable.is_pseudorange_observable() {
                    td {
                        : observable.to_string()
                    }
                }
            }
        }
        tr {
            th {
                : "Best"
            }
            @ for (observable, (_, max)) in snr_stats {
                @ if observable.is_phase_observable() || observable.is_pseudorange_observable() {
                    td {
                        b {
                            : format!("{:e}", Snr::from_str(&format!("{}", max.1)).unwrap())
                        }
                        p {
                            : format!("@{}", max.0)
                        }
                    }
                }
            }
        }
        tr {
            th {
                : "Worst"
            }
            @ for (observable, (min, _)) in snr_stats {
                @ if observable.is_phase_observable() || observable.is_pseudorange_observable() {
                    td {
                        b {
                            : format!("{:e}", Snr::from_str(&format!("{}", min.1)).unwrap())
                        }
                        p {
                            : format!("@{}", min.0)
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
            for ((epoch, _flag), (_clk, svs)) in r {
                for (_sv, observables) in svs {
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
        for ((e, _), _, obs, snr_value) in rnx.snr() {
            let snr_f64: f64 = (snr_value as u8).into();
            if let Some(values) = snr.get_mut(&obs) {
                values.push((e, snr_f64));
            } else {
                snr.insert(obs.clone(), vec![(e, snr_f64)]);
            }
        }
        /*
         * SNR analysis: {min, max}
         * per signal: we do not differentiate vehicles
         */
        let mut snr_stats: HashMap<Observable, ((Epoch, f64), (Epoch, f64))> = HashMap::new();
        for (obs, data) in snr {
            let values: Vec<f64> = data.iter().map(|(_e, value)| *value).collect();
            let min = values.clone().min();
            println!("MIN: {}", min);
            let epoch_min = data.iter().find(|(_e, value)| *value == min).unwrap().0;
            let max = values.clone().max();
            println!("MAX: {}", max);
            let epoch_max = data.iter().find(|(_e, value)| *value == max).unwrap().0;
            snr_stats.insert(obs, ((epoch_min, min), (epoch_max, max)));
        }
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
                    .map(|((e, _flag), value)| (e, value))
                    .collect();
                let der = Derivative::new(1);
                let rx_clock_drift: Vec<(Epoch, f64)> = der.eval(rx_clock);
                //TODO
                //let mov = Averager::mov(opts.clock_drift_window);
                //mov.eval(rx_clock_drift)
                rx_clock_drift
            },
        }
    }
}

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
