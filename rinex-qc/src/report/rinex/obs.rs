use itertools::Itertools;
use maud::{html, Markup, Render};
use qc_traits::{Filter, FilterItem, MaskOperand, Preprocessing};
use std::collections::HashMap;

use rinex::{
    carrier::Carrier,
    hardware::{Antenna, Receiver},
    observation::Combination,
    prelude::{Constellation, Epoch, Observable, Rinex, SV},
};

use crate::report::shared::SamplingReport;

use crate::plot::{MarkerSymbol, Mode, Plot};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Physics {
    SSI,
    Doppler,
    Phase,
    PseudoRange,
}

struct UniqueCombinationKey {
    sv: SV,
    combination: Combination,
}

impl Physics {
    pub fn from_observable(observable: &Observable) -> Self {
        if observable.is_phase_range_observable() {
            Self::Phase
        } else if observable.is_doppler_observable() {
            Self::Doppler
        } else if observable.is_ssi_observable() {
            Self::SSI
        } else {
            Self::PseudoRange
        }
    }
    pub fn plot_title(&self) -> String {
        match self {
            Self::SSI => "SSI".to_string(),
            Self::Phase => "Phase".to_string(),
            Self::Doppler => "Doppler".to_string(),
            Self::PseudoRange => "Pseudo Range".to_string(),
        }
    }
    pub fn y_label(&self) -> String {
        match self {
            Self::SSI => "Power [dB]".to_string(),
            Self::Phase => "Carrier Cycles".to_string(),
            Self::Doppler => "Doppler Shifts".to_string(),
            Self::PseudoRange => "Pseudo Range [m]".to_string(),
        }
    }
}

/// Frequency dependent pagination.
/// This depicts all information we have, on a single signal from a particular freqency.
/// This is obtained by splitting by [SV] (=signal source) and [Observable] (=freq+modulation+physics)
struct FrequencyPage {
    /// Total SPP compatible epochs
    total_spp_epochs: usize,
    /// Total CPP compatible epochs
    total_cpp_epochs: usize,
    /// Total PPP compatible epochs
    total_ppp_epochs: usize,
    /// Sampling
    sampling: SamplingReport,
    /// One plot per physics
    raw_plots: HashMap<Physics, Plot>,
    /// Code Multipath
    multipath_plot: Plot,
}

impl FrequencyPage {
    /// Builds new [FrequencyPage] from this shrinked [Rinex]
    pub fn new(rinex: &Rinex) -> Self {
        let mut total_spp_epochs = 0;
        let mut total_cpp_epochs = 0;
        let mut total_ppp_epochs = 0;

        let mut nb_pr = 0;
        let mut nb_ph = 0;
        let mut prev_t = Epoch::default();

        let sampling = SamplingReport::from_rinex(rinex);

        // Basic counters and observation analysis
        for (k, signal) in rinex.signal_observations_iter() {
            if k.epoch > prev_t {
                if nb_pr > 1 {
                    total_spp_epochs += 1;
                    total_cpp_epochs += 1;
                    if nb_ph > 1 {
                        total_ppp_epochs += 1;
                    }
                }
                nb_pr = 0;
                nb_ph = 0;
            }
            if signal.observable.is_pseudo_range_observable() {
                nb_pr += 1;
            }
            if signal.observable.is_phase_range_observable() {
                nb_ph += 1;
            }
            prev_t = k.epoch;
        }

        Self {
            sampling,
            total_cpp_epochs,
            total_spp_epochs,
            total_ppp_epochs,
            raw_plots: {
                let mut plots = HashMap::<Physics, Plot>::new();

                // draws all data points (as is).
                // Split by [SV] signal source and [Observable] (=freq;modulation;physics)
                for observable in rinex.observables_iter() {
                    // observable dependent, plot caracteristics
                    let physics = Physics::from_observable(observable);
                    let title = physics.plot_title();
                    let y_label = physics.y_label();
                    let mut plot = Plot::timedomain_plot(&title, &title, &y_label, true);

                    for sv in rinex.sv_iter() {
                        let obs_x_ok = rinex
                            .signal_observations_iter()
                            .flat_map(|(k, signal)| {
                                if k.flag.is_ok()
                                    && signal.sv == sv
                                    && &signal.observable == observable
                                {
                                    Some(k.epoch)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        let obs_y_ok = rinex
                            .signal_observations_iter()
                            .flat_map(|(k, signal)| {
                                if k.flag.is_ok()
                                    && signal.sv == sv
                                    && &signal.observable == observable
                                {
                                    Some(signal.value)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        let trace = Plot::timedomain_chart(
                            &format!("{}({})", sv, observable),
                            Mode::Markers,
                            MarkerSymbol::Cross,
                            &obs_x_ok,
                            obs_y_ok,
                            true,
                        );

                        plot.add_trace(trace);
                    }
                    plots.insert(physics, plot);
                }
                plots
            },
            multipath_plot: {
                // TODO
                Plot::timedomain_plot("code_mp", "Code Multipath", "Bias [m]", true)
            },
        }
    }
}

impl Render for FrequencyPage {
    fn render(&self) -> Markup {
        html! {
            table class="table is-bordered" {
                tr {
                    th class="is-info" {
                        "Sampling"
                    }
                    td {
                        (self.sampling.render())
                    }
                }
                tr {
                    th class="is-info" {
                        button aria-label="Epochs analysis" data-balloon-pos="right" {
                            "Epochs"
                        }
                    }
                    td {
                        table class="table is-bordered" {
                            tr {
                                th class="is-info" {
                                    button aria-label="Total SPP compatible Epochs" data-balloon-pos="right" {
                                        "SPP Compatible"
                                    }
                                }
                                td {
                                    (format!("{}/{} ({}%)", self.total_spp_epochs, self.sampling.total, self.total_spp_epochs * 100 / self.sampling.total))
                                }
                            }
                            tr {
                                th class="is-info" {
                                    button aria-label="Total CPP compatible Epochs" data-balloon-pos="right" {
                                        "CPP Compatible"
                                    }
                                }
                                td {
                                    (format!("{}/{} ({}%)", self.total_cpp_epochs, self.sampling.total, self.total_cpp_epochs * 100 / self.sampling.total))
                                }
                            }
                            tr {
                                th class="is-info" {
                                    button aria-label="Total PPP compatible Epochs" data-balloon-pos="right" {
                                        "PPP Compatible"
                                    }
                                }
                                td {
                                    (format!("{}/{} ({}%)", self.total_ppp_epochs, self.sampling.total, self.total_ppp_epochs * 100 / self.sampling.total))
                                }
                            }
                        }
                    }
                }
                tr {
                    @for physics in self.raw_plots.keys().sorted() {
                        @if let Some(plot) = self.raw_plots.get(physics) {
                            tr {
                                th class="is-info" {
                                    (format!("{} observations", physics.plot_title()))
                                }
                                td {
                                    (plot.render())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Constellation dependent pagination
struct ConstellationPage {
    /// Satellites
    satellites: Vec<SV>,
    /// sampling
    sampling: SamplingReport,
    /// True if Standard Positioning compatible
    spp_compatible: bool,
    /// True if Code Dual Frequency Positioning compatible
    cpp_compatible: bool,
    /// True if PPP compatible
    ppp_compatible: bool,
    /// Signal dependent pagination
    frequencies: HashMap<String, FrequencyPage>,
    /// SV per epoch view
    sv_epoch: HashMap<Epoch, Vec<SV>>,
    /// One plot per [Combination]
    combinations_plot: HashMap<Combination, Plot>,
    /// One plot per [Combination]
    d_dt_combinations_plot: HashMap<Combination, Plot>,
}

impl ConstellationPage {
    /// Builds new [ConstellationPage] for  this [Rinex] shrink to this [Constellation]
    pub fn new(constellation: Constellation, rinex: &Rinex) -> Self {
        // TODO
        let spp_compatible = false;
        let cpp_compatible = false;
        let ppp_compatible = false;

        let satellites = rinex.sv_iter().collect::<Vec<_>>();
        let sampling = SamplingReport::from_rinex(rinex);
        let mut frequencies = HashMap::<String, FrequencyPage>::new();

        let mut combinations_plot = HashMap::new();
        let mut d_dt_combinations_plot = HashMap::new();

        for combination in [Combination::GeometryFree, Combination::IonosphereFree] {
            let plot_id = format!("obs_{:x}", combination);
            let title = combination.to_string();

            let mut plot = Plot::timedomain_plot(&plot_id, &title, "Meters of L1-L_j delay", true);
            let mut d_dt_plot =
                Plot::timedomain_plot(&plot_id, &title, "d/dt Meters of L1-L_j delay", true);

            for sv in rinex.sv_iter() {
                let sv_filter = Filter::equals(&sv.to_string()).unwrap();

                let focused = rinex.filter(&sv_filter);

                let mut data_x = HashMap::<Observable, Vec<Epoch>>::new();
                let mut data_y = HashMap::<Observable, Vec<f64>>::new();
                let mut data_dx = HashMap::<Observable, Vec<Epoch>>::new();
                let mut data_dy = HashMap::<Observable, Vec<f64>>::new();

                for (k, value) in focused.signals_combination(combination).iter() {
                    if let Some(data_x) = data_x.get_mut(&k.reference) {
                        data_x.push(k.epoch);
                    } else {
                        data_x.insert(k.reference.clone(), vec![k.epoch]);
                    }
                    if let Some(data_y) = data_y.get_mut(&k.reference) {
                        data_y.push(*value);
                    } else {
                        data_y.insert(k.reference.clone(), vec![*value]);
                    }
                }

                for observable in data_x.keys().sorted() {
                    let data_x = data_x.get(observable).unwrap();
                    let data_y = data_y.get(observable).unwrap();
                    let tr = Plot::timedomain_chart(
                        &format!("{}({})", sv, observable),
                        Mode::Markers,
                        MarkerSymbol::Cross,
                        &data_x,
                        data_y.to_vec(),
                        true,
                    );

                    plot.add_trace(tr);
                }
            }

            combinations_plot.insert(combination, plot);
            d_dt_combinations_plot.insert(combination, d_dt_plot);
        }

        for carrier in rinex.carrier_iter() {
            let mut observables = Vec::<Observable>::new();
            for observable in rinex.observables_iter() {
                if let Ok(signal) = Carrier::from_observable(constellation, observable) {
                    if signal == carrier {
                        observables.push(observable.clone());
                    }
                }
            }
            if observables.len() > 0 {
                let filter =
                    Filter::equals(&observables.iter().map(|ob| ob.to_string()).join(", "))
                        .unwrap();
                let focused = rinex.filter(&filter);
                FrequencyPage::new(&focused);
                frequencies.insert(format!("{:?}", carrier), FrequencyPage::new(&focused));
            }
        }
        Self {
            satellites,
            sampling,
            frequencies,
            spp_compatible,
            cpp_compatible,
            ppp_compatible,
            sv_epoch: Default::default(),
            combinations_plot,
            d_dt_combinations_plot,
        }
    }
}

impl Render for ConstellationPage {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th {
                                button aria-label="Pseudo Range single frequency navigation" data-balloon-pos="right" {
                                    "SPP Compatible"
                                }
                            }
                            td {
                                @if self.spp_compatible {
                                    span class="icon" style="color:green" {
                                        i class="fa-solid fa-circle-check" {}
                                    }
                                } @else {
                                    span class="icon" style="color:red" {
                                        i class="fa-solid fa-circle-xmark" {}
                                    }
                                }
                            }
                        }
                        tr {
                            th {
                                button aria-label="Pseudo Range dual frequency navigation" data-balloon-pos="right" {
                                    "CPP compatible"
                                }
                            }
                            td {
                                @if self.cpp_compatible {
                                    span class="icon" style="color:green" {
                                        i class="fa-solid fa-circle-check" {}
                                    }
                                } @else {
                                    span class="icon" style="color:red" {
                                        i class="fa-solid fa-circle-xmark" {}
                                    }
                                }
                            }
                        }
                        tr {
                            th {
                                button aria-label="Dual frequency Pseudo + Phase Range navigation" data-balloon-pos="right" {
                                    "PPP compatible"
                                }
                            }
                            td {
                                @if self.ppp_compatible {
                                    span class="icon" style="color:green" {
                                        i class="fa-solid fa-circle-check" {};
                                    }
                                } @else {
                                    span class="icon" style="color:red" {
                                        i class="fa-solid fa-circle-xmark" {};
                                    }
                                }
                            }
                        }
                        tr {
                            th class="is-info" {
                                button aria-label="Observed Satellites" data-balloon-pos="right" {
                                    "Satellites"
                                }
                            }
                            td {
                                (self.satellites.iter().sorted().join(", "))
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Sampling"
                            }
                            td {
                                (self.sampling.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Signals"
                            }
                            td {
                                (self.frequencies.keys().sorted().join(", "))
                            }
                        }
                        @for combination in self.combinations_plot.keys().sorted() {
                            @if let Some(plot) = self.combinations_plot.get(combination) {
                                tr {
                                    th class="is-info" {
                                        (format!("{} combination", combination))
                                    }
                                    td {
                                        (plot.render())
                                    }
                                }
                            }
                            @if let Some(plot) = self.d_dt_combinations_plot.get(combination) {
                                tr {
                                    th class="is-info" {
                                        (format!("d/dt {} combination", combination))
                                    }
                                    td {
                                        (plot.render())
                                    }
                                }
                            }
                        }
                        @for signal in self.frequencies.keys().sorted() {
                            @if let Some(page) = self.frequencies.get(signal) {
                                tr {
                                    th class="is-info" {
                                        (signal.to_string())
                                    }
                                    td id=(format!("page:obs:{}", signal)) class="page:obs:{}" style="display:block" {
                                        (page.render())
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// RINEX Observation Report
pub struct Report {
    antenna: Option<Antenna>,
    receiver: Option<Receiver>,
    clock_plot: Plot,
    sampling: SamplingReport,
    constellations: HashMap<String, ConstellationPage>,
}

impl Report {
    /// Rends Tabbed menu bar as [Markup]
    pub fn html_inline_menu_bar(&self) -> Markup {
        html! {
            a id="menu:obs" {
                span class="icon" {
                    i class="fa-solid fa-tower-cell" {}
                }
                "Observations"
            }
            ul class="menu-list" style="display:block" {
                @for constell in self.constellations.keys().sorted() {
                    li {
                        a id=(&format!("menu:obs:{}", constell)) class="menu:subtab" style="margin-left:29px" {
                            span class="icon" {
                                i class="fa-solid fa-satellite" {};
                            }
                            (constell.to_string())
                        }
                    }
                }
            }
        }
    }

    /// Builds new [Report] from this [Rinex]
    pub fn new(rinex: &Rinex) -> Self {
        Self {
            sampling: SamplingReport::from_rinex(rinex),
            receiver: if let Some(rcvr) = &rinex.header.rcvr {
                Some(rcvr.clone())
            } else {
                None
            },
            antenna: if let Some(ant) = &rinex.header.rcvr_antenna {
                Some(ant.clone())
            } else {
                None
            },
            clock_plot: {
                let mut plot = Plot::timedomain_plot("rx_clock", "Clock offset", "Second", true);
                plot
            },
            constellations: {
                let mut constellations = HashMap::<String, ConstellationPage>::new();
                for constellation in rinex.constellations_iter() {
                    let filter = Filter::mask(
                        MaskOperand::Equals,
                        FilterItem::ConstellationItem(vec![constellation]),
                    );
                    //if constellation == Constellation::BeiDou {
                    //    // MEO mask
                    //    let meo1 = Filter::greater_than("C05").unwrap();
                    //    let meo2 = Filter::lower_than("C58").unwrap();
                    //    let meo = rinex.filter(&meo1).filter(&meo2);

                    //    constellations.insert(
                    //        "BeiDou (MEO)".to_string(),
                    //        ConstellationPage::new(constellation, &meo),
                    //    );

                    //    // GEO mask
                    //    let geo = rinex.filter(&!meo1).filter(&!meo2);

                    //    constellations.insert(
                    //        "BeiDou (GEO)".to_string(),
                    //        ConstellationPage::new(constellation, &geo),
                    //    );
                    //} else {
                    let focused = rinex.filter(&filter);
                    constellations.insert(
                        constellation.to_string(),
                        ConstellationPage::new(constellation, &focused),
                    );
                    //}
                }
                constellations
            },
        }
    }
}

impl Render for Report {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                @if let Some(rx) = &self.receiver {
                    table class="table is-bordered" {
                        tr {
                            th class="is-info" {
                                "Receiver"
                            }
                            td {
                                (rx.render())
                            }
                        }
                    }
                }
                @if let Some(ant) = &self.antenna {
                    table class="table is-bordered" {
                        tr {
                            th class="is-info" {
                                "Antenna"
                            }
                            td {
                                 (ant.render())
                            }
                        }
                    }
                }
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "Sampling"
                        }
                        td {
                            (self.sampling.render())
                        }
                    }
                }
                @for constell in self.constellations.keys().sorted() {
                    @if let Some(page) = self.constellations.get(constell) {
                        table class="table is-bordered is-page" id=(format!("body:obs:{}", constell)) style="display:none" {
                            tr {
                                th class="is-info" {
                                    (constell.to_string())
                                }
                                td {
                                    (page.render())
                                }
                            }
                        }
                    }
                }
            }//table-container
        }
    }
}
