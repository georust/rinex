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

/// Constellation dependent pagination
pub struct ConstellationCombination {
    /// GF plot
    gf_plot: Plot,
    /// d/dt GF plot
    d_dt_gf_plot: Plot,
    /// d/dt IF plot
    d_dt_if_plot: Plot,
    /// MP plot
    mp_plot: Plot,
}

impl ConstellationCombination {
    /// Builds new [ConstellationPage] for  this [Rinex] shrink to this [Constellation]
    pub fn new(constellation: Constellation, rinex: &Rinex) -> Self {
        let mut gf_plot = Plot::timedomain_plot(
            "obs_gf_plot",
            "Geometry Free",
            "L1-L_j meters delay (bias)",
            true,
        );

        let mut d_dt_gf_plot = Plot::timedomain_plot(
            "obs_d_dt_gf_plot",
            "Geometry Free Variations",
            "d/dt(bias)",
            true,
        );

        let mut d_dt_if_plot = Plot::timedomain_plot(
            "obs_gf_plot",
            "Ionosphere Free variations",
            "d/dt(bias)",
            true,
        );

        for combination in [Combination::GeometryFree, Combination::IonosphereFree] {
            for (sv_index, sv) in rinex.sv_iter().enumerate() {
                let sv_filter = Filter::equals(&sv.to_string()).unwrap();

                let focused = rinex.filter(&sv_filter);

                let mut data_x = HashMap::<Observable, Vec<Epoch>>::new();
                let mut data_y = HashMap::<Observable, Vec<f64>>::new();

                let mut prev = HashMap::<Observable, (Epoch, f64)>::new();
                let mut data_dx = HashMap::<Observable, Vec<Epoch>>::new();
                let mut data_dy = HashMap::<Observable, Vec<f64>>::new();

                for (k, value) in focused.signals_combination(combination).iter() {
                    if let Some(data_x) = data_x.get_mut(&k.lhs) {
                        data_x.push(k.epoch);
                    } else {
                        data_x.insert(k.lhs.clone(), vec![k.epoch]);
                    }
                    if let Some(data_y) = data_y.get_mut(&k.lhs) {
                        data_y.push(*value);
                    } else {
                        data_y.insert(k.lhs.clone(), vec![*value]);
                    }
                    if let Some((prev_t, prev_y)) = prev.get_mut(&k.lhs) {
                        if let Some(data_x) = data_dx.get_mut(&k.lhs) {
                            data_x.push(k.epoch);
                        } else {
                            data_dx.insert(k.lhs.clone(), vec![k.epoch]);
                        }

                        let dt = (k.epoch - *prev_t).to_seconds();
                        let dy = *value - *prev_y;

                        if let Some(data_y) = data_dy.get_mut(&k.lhs) {
                            data_y.push(dy / dt);
                        } else {
                            data_dy.insert(k.lhs.clone(), vec![dy / dt]);
                        }

                        *prev_t = k.epoch;
                        *prev_y = *value;
                    } else {
                        prev.insert(k.lhs.clone(), (k.epoch, *value));
                    }
                }

                for (obs_index, observable) in data_x.keys().sorted().enumerate() {
                    let data_x = data_x.get(observable).unwrap();
                    let data_y = data_y.get(observable).unwrap();

                    let data_dx = data_dx.get(observable).unwrap();
                    let data_dy = data_dy.get(observable).unwrap();

                    let tr = Plot::timedomain_chart(
                        &format!("{}({})", sv, observable),
                        Mode::Markers,
                        MarkerSymbol::Cross,
                        &data_dx,
                        data_dy.to_vec(),
                        sv_index == 0 && obs_index == 0,
                    );

                    match combination {
                        Combination::GeometryFree => {
                            d_dt_gf_plot.add_trace(tr);

                            let tr = Plot::timedomain_chart(
                                &format!("{}({})", sv, observable),
                                Mode::Markers,
                                MarkerSymbol::Cross,
                                &data_x,
                                data_y.to_vec(),
                                sv_index == 0 && obs_index == 0,
                            );

                            gf_plot.add_trace(tr);
                        },
                        Combination::IonosphereFree => {
                            d_dt_if_plot.add_trace(tr);
                        },
                        _ => {},
                    }
                }
            }
        }

        let mut mp_plot = Plot::timedomain_plot("obs_mp", "Multipath Bias", "Bias [m]", true);

        for (sv_index, sv) in rinex.sv_iter().enumerate() {
            let filter = Filter::equals(&sv.to_string()).unwrap();

            let focused = rinex.filter(&filter);

            let mut data_x = HashMap::<(Observable, Observable), Vec<Epoch>>::new();
            let mut data_y = HashMap::<(Observable, Observable), Vec<f64>>::new();

            for (k, v) in focused.signals_multipath().iter() {
                if let Some(data_x) = data_x.get_mut(&(k.signal.clone(), k.rhs.clone())) {
                    data_x.push(k.epoch);
                } else {
                    data_x.insert((k.signal.clone(), k.rhs.clone()), vec![k.epoch]);
                }
                if let Some(data_y) = data_y.get_mut(&(k.signal.clone(), k.rhs.clone())) {
                    data_y.push(*v);
                } else {
                    data_y.insert((k.signal.clone(), k.rhs.clone()), vec![*v]);
                }
            }

            for (mp_index, ((ref_obs, rhs_obs), data_x)) in data_x.iter().enumerate() {
                let data_y = data_y.get(&(ref_obs.clone(), rhs_obs.clone())).unwrap();

                let tr = Plot::timedomain_chart(
                    &format!("{}({}/{})", sv, ref_obs, rhs_obs),
                    Mode::Markers,
                    MarkerSymbol::Cross,
                    data_x,
                    data_y.to_vec(),
                    sv_index == 0 && mp_index == 0,
                );

                mp_plot.add_trace(tr);
            }
        }

        Self {
            mp_plot,
            gf_plot,
            d_dt_gf_plot,
            d_dt_if_plot,
        }
    }
}

impl Render for ConstellationCombination {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Geometry Free combination"
                            }
                            td {
                                (self.gf_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Geometry Free variations"
                            }
                            td {
                                (self.d_dt_gf_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Ionosphere Free combination"
                            }
                            td {
                                (self.d_dt_if_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Signal Multipath"
                            }
                            td id="obs_mp" {
                                (self.mp_plot.render())
                            }
                        }
                    }
                }
            }
        }
    }
}

/// RINEX Observation Report shared by both ROVERs and BASEs
pub struct QcSignalCombinationsReport {
    pub constellations: HashMap<String, ConstellationCombination>,
}

impl QcSignalCombinationsReport {
    /// Rends Tabbed menu bar as [Markup]
    pub fn html_inline_menu_bar(&self) -> Markup {
        html! {
            a {
                span class="icon" {
                    i class="fa-solid fa-tower-cell" {}
                }
                "Signal Combinations"
            }
        }
    }

    /// Builds new [Report] from this [Rinex]
    pub fn new(rinex: &Rinex) -> Self {
        Self {
            constellations: HashMap::new(),
        }
    }
}

impl Render for QcSignalCombinationsReport {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                @for constell in self.constellations.keys().sorted() {
                    @if let Some(combinations) = self.constellations.get(constell) {
                        tr {
                            th class="is-info" {
                                (constell.to_string())
                            }
                            td {
                                (combinations.render())
                            }
                        }
                    }
                }
            }//table-container
        }
    }
}
