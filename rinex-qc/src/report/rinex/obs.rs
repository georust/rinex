use itertools::Itertools;
use maud::{html, Markup, Render};
use qc_traits::{Filter, FilterItem, MaskOperand, Preprocessing};
use std::collections::HashMap;

use rinex::{
    carrier::Carrier,
    hardware::{Antenna, Receiver},
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

struct Combination {
    lhs: Observable,
    rhs: Observable,
}

/// Frequency dependent pagination
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
    /// One plot per combination,
    combination_plots: HashMap<Combination, Plot>,
    /// Code Multipath
    multipath_plot: Plot,
}

impl FrequencyPage {
    pub fn new(rinex: &Rinex) -> Self {
        let mut total_spp_epochs = 0;
        let mut total_cpp_epochs = 0;
        let mut total_ppp_epochs = 0;
        let sampling = SamplingReport::from_rinex(rinex);

        for (k, v) in rinex.observations_iter() {
            let mut nb_pr = 0;
            let mut nb_ph = 0;

            for signal in v.signals.iter() {
                // if nb_pr > 0 {
                //     total_spp_epochs += 1;
                // }
                // if nb_pr > 1 {
                //     total_cpp_epochs += 1;
                //     if nb_ph > 1 {
                //         total_ppp_epochs += 1;
                //     }
                // }
            }
        }

        Self {
            sampling,
            total_cpp_epochs,
            total_spp_epochs,
            total_ppp_epochs,
            combination_plots: HashMap::new(),
            multipath_plot: Plot::timedomain_plot("code_mp", "Code Multipath", "Bias [m]", true),
            raw_plots: {
                let mut plots = HashMap::<Physics, Plot>::new();
                let svnn = rinex.sv_iter().collect::<Vec<_>>();
                let observables = rinex.observables_iter().collect::<Vec<_>>();

                // draw carrier phase plot for all SV; per signal

                for obs in observables {
                    let physics = Physics::from_observable(obs);
                    let title = physics.plot_title();
                    let y_label = physics.y_label();
                    let mut plot = Plot::timedomain_plot(&title, &title, &y_label, true);

                    for sv in &svnn {
                        let obs_x_ok = rinex
                            .signal_observations_iter()
                            .flat_map(|(k, v)| {
                                if v.sv == *sv && v.observable == *obs {
                                    Some(k.epoch)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        let obs_y_ok = rinex
                            .signal_observations_iter()
                            .flat_map(|(k, v)| {
                                if v.sv == *sv && v.observable == *obs {
                                    Some(v.value)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        let trace = Plot::timedomain_chart(
                            &format!("{}({})", sv, obs),
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
    /// SV per epoch
    sv_epoch: HashMap<Epoch, Vec<SV>>,
}

impl ConstellationPage {
    pub fn new(constellation: Constellation, rinex: &Rinex) -> Self {
        let mut spp_compatible = false; // TODO
        let mut cpp_compatible = false; // TODO
        let mut ppp_compatible = false; // TODO
        let satellites = rinex.sv_iter().collect::<Vec<_>>();
        let sampling = SamplingReport::from_rinex(rinex);
        let mut frequencies = HashMap::<String, FrequencyPage>::new();
        for carrier in rinex.carrier_iter().sorted() {
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
                let mut constellations: HashMap<String, ConstellationPage> =
                    HashMap::<String, ConstellationPage>::new();
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
                            // TODO
                            // td {
                            //     (rx.render())
                            // }
                        }
                    }
                }
                @if let Some(ant) = &self.antenna {
                    table class="table is-bordered" {
                        tr {
                            th class="is-info" {
                                "Antenna"
                            }
                            // TODO
                            // td {
                            //      (ant.render())
                            // }
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
