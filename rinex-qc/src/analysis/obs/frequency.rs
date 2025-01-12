//! Frequency pagination during signal analysis

use std::collections::HashMap;

use crate::{prelude::Plot, report::obs::Physics};

use rinex::prelude::{Epoch, Rinex};

/// This depicts all information we have, on a single signal from a particular freqency.
/// This is obtained by splitting by [SV] (=signal source) and [Observable] (=freq+modulation+physics)
pub struct FrequencyPage {
    /// One plot per physics
    raw_plots: HashMap<Physics, Plot>,
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
