use itertools::Itertools;
use maud::{html, Markup, Render};
use qc_traits::processing::{Filter, FilterItem, MaskOperand, Preprocessing};
use std::collections::HashMap;

use rinex::{
    carrier::Carrier,
    hardware::{Antenna, Receiver},
    prelude::{Constellation, Duration, Epoch, Observable, Rinex, SV},
};

use crate::report::shared::{EpochSlider, SamplingReport};

#[cfg(feature = "plot")]
use crate::plot::{MarkerSymbol, Mode, Plot};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "plot")]
enum Physics {
    SSI,
    Doppler,
    Phase,
    PseudoRange,
}

#[cfg(feature = "plot")]
impl Physics {
    pub fn from_observable(observable: &Observable) -> Self {
        if observable.is_phase_observable() {
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
    #[cfg(feature = "plot")]
    /// One plot per physics
    raw_plots: HashMap<Physics, Plot>,
    #[cfg(feature = "plot")]
    /// One plot per combination,
    combination_plots: HashMap<Combination, Plot>,
    #[cfg(feature = "plot")]
    /// Code Multipath
    multipath_plot: Plot,
}

impl FrequencyPage {
    pub fn new(rinex: &Rinex) -> Self {
        let mut total_spp_epochs = 0;
        let mut total_cpp_epochs = 0;
        let mut total_ppp_epochs = 0;
        let sampling = SamplingReport::from_rinex(rinex);
        for (_, (_, svnn)) in rinex.observation() {
            let mut nb_pr = 0;
            let mut nb_ph = 0;
            for (_, obs) in svnn.iter() {}
            if nb_pr > 0 {
                total_spp_epochs += 1;
            }
            if nb_pr > 1 {
                total_cpp_epochs += 1;
                if nb_ph > 1 {
                    total_ppp_epochs += 1;
                }
            }
        }
        Self {
            sampling,
            total_cpp_epochs,
            total_spp_epochs,
            total_ppp_epochs,
            #[cfg(feature = "plot")]
            combination_plots: HashMap::new(),
            #[cfg(feature = "plot")]
            multipath_plot: Plot::timedomain_plot("code_mp", "Code Multipath", "Bias [m]", true),
            #[cfg(feature = "plot")]
            raw_plots: {
                let mut plots = HashMap::<Physics, Plot>::new();
                let svnn = rinex.sv().collect::<Vec<_>>();
                let observables = rinex.observable().collect::<Vec<_>>();
                // draw carrier phase plot for all SV; per signal
                for ob in observables {
                    let physics = Physics::from_observable(ob);
                    let title = physics.plot_title();
                    let y_label = physics.y_label();
                    let mut plot = Plot::timedomain_plot(&title, &title, &y_label, true);
                    for sv in &svnn {
                        let obs_x_ok = rinex
                            .observation()
                            .flat_map(|((t, flag), (_, svnn))| {
                                svnn.iter().flat_map(move |(svnn, observations)| {
                                    observations.iter().filter_map(move |(obs, value)| {
                                        if ob == obs && flag.is_ok() && svnn == sv {
                                            Some(*t)
                                        } else {
                                            None
                                        }
                                    })
                                })
                            })
                            .collect::<Vec<_>>();
                        let obs_y_ok = rinex
                            .observation()
                            .flat_map(|((t, flag), (_, svnn))| {
                                svnn.iter().flat_map(move |(svnn, observations)| {
                                    observations.iter().filter_map(move |(obs, value)| {
                                        if ob == obs && flag.is_ok() && svnn == sv {
                                            Some(value.obs)
                                        } else {
                                            None
                                        }
                                    })
                                })
                            })
                            .collect::<Vec<_>>();
                        let trace = Plot::timedomain_chart(
                            &format!("{}({})", sv, ob),
                            Mode::Markers,
                            MarkerSymbol::Cross,
                            &obs_x_ok,
                            obs_y_ok,
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
                        "Epochs"
                    }
                    td {
                        table class="table is-bordered" {
                            tr {
                                th class="is-info" {
                                    "SPP Compatible"
                                }
                                td {
                                    (format!("{}/{} ({}%)", self.total_spp_epochs, self.sampling.total, self.total_spp_epochs * 100 / self.sampling.total))
                                }
                            }
                            tr {
                                th class="is-info" {
                                    "CPP Compatible"
                                }
                                td {
                                    (format!("{}/{} ({}%)", self.total_cpp_epochs, self.sampling.total, self.total_cpp_epochs * 100 / self.sampling.total))
                                }
                            }
                            tr {
                                th class="is-info" {
                                    "PPP Compatible"
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
    /// List of SV
    sv: Vec<SV>,
    /// First epoch
    first_epoch: Epoch,
    /// Last epoch
    last_epoch: Epoch,
    /// Timeframe
    duration: Duration,
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
        let sv_list = rinex.sv().collect::<Vec<_>>();
        let first_epoch = rinex.first_epoch().unwrap_or_default();
        let last_epoch = rinex.last_epoch().unwrap_or_default();
        let mut frequencies = HashMap::<String, FrequencyPage>::new();
        for carrier in rinex.carrier().sorted() {
            let mut observables = Vec::<Observable>::new();
            for observable in rinex.observable() {
                if Carrier::from_observable(constellation, observable).is_ok() {
                    observables.push(observable.clone());
                }
            }
            let filter = Filter::mask(
                MaskOperand::Equals,
                FilterItem::ComplexItem(
                    observables
                        .iter()
                        .map(|ob| ob.to_string())
                        .collect::<Vec<_>>(),
                ),
            );
            let focused = rinex.filter(&filter);
            FrequencyPage::new(&focused);
            frequencies.insert(format!("{:?}", carrier), FrequencyPage::new(&focused));
        }
        Self {
            sv: sv_list,
            frequencies,
            spp_compatible,
            cpp_compatible,
            ppp_compatible,
            sv_epoch: rinex.sv_epoch().collect(),
            first_epoch,
            last_epoch,
            duration: last_epoch - first_epoch,
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
                                "SPP Compatible"
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
                                "CPP compatible"
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
                                "PPP compatible"
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
                                "SV"
                            }
                            td {
                                (self.sv.iter().sorted().join(", "))
                            }
                        }
                        @if !self.sv_epoch.is_empty() {
                            @let slider = EpochSlider::new(
                                self.first_epoch,
                                self.last_epoch,
                                self.duration,
                            );
                            tr {
                                th {
                                    (slider.render())
                                }
                                td id="sv_epoch" {
                                    (self.sv_epoch.get(&self.first_epoch).unwrap().iter().join(", "))
                                }
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
    sampling: SamplingReport,
    #[cfg(feature = "plot")]
    clock_plot: Plot,
    constellations: HashMap<Constellation, ConstellationPage>,
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
            #[cfg(feature = "plot")]
            clock_plot: {
                let mut plot = Plot::timedomain_plot("rx_clock", "Clock offset", "Second", true);
                plot
            },
            constellations: {
                let mut constellations = HashMap::<Constellation, ConstellationPage>::new();
                for constellation in rinex.constellation() {
                    let filter = Filter::mask(
                        MaskOperand::Equals,
                        FilterItem::ConstellationItem(vec![constellation]),
                    );
                    let focused = rinex.filter(&filter);
                    constellations.insert(
                        constellation,
                        ConstellationPage::new(constellation, &focused),
                    );
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
