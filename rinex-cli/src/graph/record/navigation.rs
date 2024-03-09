use crate::graph::{build_3d_chart_epoch_label, build_chart_epoch_axis, PlotContext};
use plotly::common::{Mode, Visible};
use rinex::navigation::Ephemeris;
use rinex::prelude::*;

use itertools::Itertools;
use std::collections::{BTreeMap, HashMap};

use sp3::SP3;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ProductType {
    Radio,
    HighPrecisionSp3,
    HighPrecisionClk,
}

/// Clock states from all products provided by User
type CtxClockStates = HashMap<ProductType, BTreeMap<SV, Vec<(Epoch, (f64, f64, f64))>>>;
/// Clock Corrections from all products provided by User
type CtxClockCorrections = HashMap<ProductType, BTreeMap<SV, Vec<(Epoch, Duration)>>>;

pub fn plot_sv_nav_clock(ctx: &RnxContext, plot_ctx: &mut PlotContext) {
    if let Some(nav) = ctx.brdc_navigation() {
        let nav_sv = nav.sv().collect::<Vec<_>>();
        let clk = ctx.clock();
        let sp3 = ctx.sp3();
        let clock_states = ctx_sv_clock_states(nav, &nav_sv, clk, sp3);
        plot_sv_clock_states(&clock_states, &nav_sv, plot_ctx);

        if let Some(obs) = ctx.observation() {
            let clock_corrections = ctx_sv_clock_corrections(obs, nav, clk, sp3);
            plot_sv_clock_corrections(&clock_corrections, plot_ctx);
            plot_system_time(&clock_states, &clock_corrections, plot_ctx);
        } else {
            info!("adding OBS RINEX will provide clock corrections graphs");
        }
    } else {
        warn!("missing navigation (orbits) data");
    }
}

/*
 * Determines all clock states
 */
fn ctx_sv_clock_states(
    nav: &Rinex,
    nav_sv: &Vec<SV>,
    clk: Option<&Rinex>,
    sp3: Option<&SP3>,
) -> CtxClockStates {
    let mut states = CtxClockStates::new();
    for product in [
        ProductType::Radio,
        ProductType::HighPrecisionSp3,
        ProductType::HighPrecisionClk,
    ] {
        match product {
            ProductType::Radio => {
                let mut tree = BTreeMap::<SV, Vec<(Epoch, (f64, f64, f64))>>::new();
                for (t, sv, (bias, drift, driftr)) in nav.sv_clock() {
                    tree.entry(sv).or_default().push((t, (bias, drift, driftr)));
                }
                states.insert(product, tree);
            },
            ProductType::HighPrecisionSp3 => {
                if let Some(sp3) = sp3 {
                    let mut tree = BTreeMap::<SV, Vec<(Epoch, (f64, f64, f64))>>::new();
                    for (t, sv, bias) in sp3.sv_clock() {
                        if !nav_sv.contains(&sv) {
                            continue;
                        }
                        tree.entry(sv)
                            .or_default()
                            .push((t, (bias, 0.0_f64, 0.0_f64)));
                    }
                    // TODO: augment clock offset with possible drift
                    //for (t, sv, drift) in sp3.sv_clock_change() {
                    //    if !nav_sv.contains(&sv) {
                    //        continue;
                    //    }
                    //    if let Some(inner) = tree.get_mut(&sv) {
                    //        //TODO augment with drift
                    //    } else {
                    //        tree.insert(sv, vec![(t, (0.0_f64, drift, 0.0_f64))]);
                    //    }
                    //}
                    states.insert(product, tree);
                }
            },
            ProductType::HighPrecisionClk => {
                if let Some(clk) = clk {
                    let mut tree = BTreeMap::<SV, Vec<(Epoch, (f64, f64, f64))>>::new();
                    for (t, sv, _, profile) in clk.precise_sv_clock() {
                        if !nav_sv.contains(&sv) {
                            continue;
                        }
                        let ck = (
                            profile.bias,
                            profile.drift.unwrap_or(0.0_f64),
                            profile.drift_change.unwrap_or(0.0_f64),
                        );
                        tree.entry(sv).or_default().push((t, ck));
                    }
                    states.insert(product, tree);
                }
            },
        }
    }
    states
}

/*
 * Plot SV Clock Offset/Drift
 * one plot (2 Y axes) for both Clock biases
 * and clock drift
 */
fn plot_sv_clock_states(ctx: &CtxClockStates, nav_sv: &Vec<SV>, plot_ctx: &mut PlotContext) {
    trace!("sv clock states plot");
    for (product, vehicles) in ctx {
        match product {
            ProductType::Radio => {
                plot_ctx.add_timedomain_2y_plot("BRDC SV Clock", "Offset [s]", "Drift [s/s]");
            },
            ProductType::HighPrecisionSp3 => {
                plot_ctx.add_timedomain_2y_plot("SP3 SV Clock", "Offset [s]", "Drift [s/s]");
            },
            ProductType::HighPrecisionClk => {
                plot_ctx.add_timedomain_2y_plot("CLK SV Clock", "Offset [s]", "Drift [s/s]");
            },
        };
        for (index, (sv, results)) in vehicles.iter().enumerate() {
            if !nav_sv.contains(&sv) {
                continue;
            }
            let sv_epochs = results.iter().map(|(t, _)| *t).collect::<Vec<_>>();
            let sv_bias = results
                .iter()
                .map(|(_, (bias, _, _))| *bias)
                .collect::<Vec<_>>();
            let sv_drift = results
                .iter()
                .map(|(_, (_, drift, _))| *drift)
                .collect::<Vec<_>>();

            let trace = build_chart_epoch_axis(
                &format!("{:X}(offset)", sv),
                Mode::LinesMarkers,
                sv_epochs.clone(),
                sv_bias,
            )
            .visible({
                if index == 0 {
                    /*
                     * Clock data differs too much,
                     * looks better if we only present one by default
                     */
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
            plot_ctx.add_trace(trace);

            let trace = build_chart_epoch_axis(
                &format!("{:X}(drift)", sv),
                Mode::LinesMarkers,
                sv_epochs.clone(),
                sv_drift,
            )
            .y_axis("y2")
            .visible({
                if index == 0 {
                    /*
                     * Clock data differs too much,
                     * looks better if we only present one by default
                     */
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
            plot_ctx.add_trace(trace);
        }
    }
}

/*
/ * Determines all clock corrections from context
/ */
fn ctx_sv_clock_corrections(
    obs: &Rinex,
    nav: &Rinex,
    clk: Option<&Rinex>,
    sp3: Option<&SP3>,
) -> CtxClockCorrections {
    let mut clock_corr = CtxClockCorrections::new();
    for ((t, flag), (_, vehicles)) in obs.observation() {
        if !flag.is_ok() {
            continue;
        }
        for (sv, _) in vehicles {
            let sv_eph = nav.sv_ephemeris(*sv, *t);
            if sv_eph.is_none() {
                continue;
            }

            let (toe, sv_eph) = sv_eph.unwrap();

            for product in [
                ProductType::Radio,
                ProductType::HighPrecisionSp3,
                ProductType::HighPrecisionClk,
            ] {
                let clock_state: Option<(f64, f64, f64)> = match product {
                    ProductType::Radio => Some(sv_eph.sv_clock()),
                    ProductType::HighPrecisionSp3 => {
                        //TODO: sv_clock interpolate please
                        None
                    },
                    ProductType::HighPrecisionClk => {
                        if let Some(clk) = clk {
                            clk.precise_sv_clock()
                                .filter_map(|(clk_t, clk_sv, _, prof)| {
                                    if clk_t == *t && clk_sv == *sv {
                                        Some((
                                            prof.bias,
                                            prof.drift.unwrap_or(0.0_f64),
                                            prof.drift_change.unwrap_or(0.0_f64),
                                        ))
                                    } else {
                                        None
                                    }
                                })
                                .reduce(|k, _| k)
                            //if let Some((_, profile)) = clk
                            //    .precise_sv_clock_interpolate(*t, *sv)
                            //{
                            //    Some((profile.bias, profile.drift.unwrap_or(0.0_f64), profile.drift_change.unwrap_or(0.0_f64)))
                            //} else {
                            //    None
                            //}
                        } else {
                            None
                        }
                    },
                };

                if let Some(clock_state) = clock_state {
                    let correction = Ephemeris::sv_clock_corr(*sv, clock_state, *t, toe);

                    // insert result
                    if let Some(inner) = clock_corr.get_mut(&product) {
                        if let Some(inner) = inner.get_mut(&sv) {
                            inner.push((*t, correction));
                        } else {
                            inner.insert(*sv, vec![(*t, correction)]);
                        }
                    } else {
                        let mut inner = BTreeMap::<SV, Vec<(Epoch, Duration)>>::new();
                        inner.insert(*sv, vec![(*t, correction)]);
                        clock_corr.insert(product, inner);
                    }
                }
            }
        }
    }
    clock_corr
}

/*
/ * Plot SV clock corrections
*/
fn plot_sv_clock_corrections(ctx: &CtxClockCorrections, plot_ctx: &mut PlotContext) {
    trace!("sv clock corrections plot");
    for (product, vehicles) in ctx {
        for ts in vehicles
            .iter()
            .map(|(sv, _)| sv.constellation.timescale().unwrap())
            .unique()
        {
            match product {
                ProductType::Radio => {
                    plot_ctx.add_timedomain_plot(
                        &format!("|{} - SV| BRDC Correction", ts),
                        "Offset [s]",
                    );
                },
                ProductType::HighPrecisionSp3 => {
                    plot_ctx.add_timedomain_plot(
                        &format!("|{} - SV| SP3 Correction", ts),
                        "Offset [s]",
                    );
                },
                ProductType::HighPrecisionClk => {
                    plot_ctx.add_timedomain_plot(
                        &format!("|{} - SV| CLK Correction", ts),
                        "Offset [s]",
                    );
                },
            }
            for (index, (sv, data)) in vehicles
                .iter()
                .filter(|(sv, data)| sv.constellation.timescale().unwrap() == ts)
                .enumerate()
            {
                let epochs = data.iter().map(|(t, _)| *t).collect::<Vec<_>>();
                let offset = data
                    .iter()
                    .map(|(_, dt)| dt.to_seconds())
                    .collect::<Vec<_>>();
                let trace =
                    build_chart_epoch_axis(&format!("{}", sv), Mode::Markers, epochs, offset)
                        .visible({
                            if index == 0 {
                                Visible::True
                            } else {
                                Visible::LegendOnly
                            }
                        });
                plot_ctx.add_trace(trace);
            }
        }
    }
}

/*
 * Plot System time as resolved from all provided products
 */
fn plot_system_time(
    states: &CtxClockStates,
    corrections: &CtxClockCorrections,
    plot_ctx: &mut PlotContext,
) {
    trace!("time system plot");
    for (product, vehicles) in corrections {
        for ts in vehicles
            .iter()
            .map(|(sv, _)| sv.constellation.timescale().unwrap())
            .unique()
        {
            match product {
                ProductType::Radio => {
                    plot_ctx.add_timedomain_plot(&format!("{} from BRDC", ts), "Epoch");
                },
                ProductType::HighPrecisionSp3 => {
                    plot_ctx.add_timedomain_plot(&format!("{} from SP3", ts), "Epoch");
                },
                ProductType::HighPrecisionClk => {
                    plot_ctx.add_timedomain_plot(&format!("{} from CLK", ts), "Epoch");
                },
            }
            for (_, state_vehicles) in states.iter().filter(|(k, _)| *k == product) {
                for (index, (sv, corrections)) in vehicles
                    .iter()
                    .filter(|(sv, data)| sv.constellation.timescale() == Some(ts))
                    .enumerate()
                {
                    if let Some((_, data)) = state_vehicles
                        .iter()
                        .filter(|(state_sv, data)| *state_sv == sv)
                        .reduce(|k, _| k)
                    {
                        //FIXME: conclude this graph
                    }
                }
            }
        }
    }
}

pub fn plot_sv_nav_orbits(ctx: &RnxContext, plot_ctx: &mut PlotContext) {
    let mut pos_plot_created = false;
    let mut nav_sv = Vec::<SV>::with_capacity(32);
    /*
     * Plot Broadcast Orbit (x, y, z)
     */
    if let Some(nav) = ctx.brdc_navigation() {
        for (sv_index, sv) in nav.sv().enumerate() {
            nav_sv.push(sv);
            if sv_index == 0 {
                plot_ctx.add_cartesian3d_plot("SV Orbit (broadcast)", "x [km]", "y [km]", "z [km]");
                trace!("broadcast orbit plot");
                pos_plot_created = true;
            }
            let epochs: Vec<_> = nav
                .sv_position()
                .filter_map(
                    |(epoch, svnn, (_, _, _))| {
                        if svnn == sv {
                            Some(epoch)
                        } else {
                            None
                        }
                    },
                )
                .collect();

            let x_km: Vec<_> = nav
                .sv_position()
                .filter_map(
                    |(_epoch, svnn, (x, _, _))| {
                        if svnn == sv {
                            Some(x)
                        } else {
                            None
                        }
                    },
                )
                .collect();
            let y_km: Vec<_> = nav
                .sv_position()
                .filter_map(
                    |(_epoch, svnn, (_, y, _))| {
                        if svnn == sv {
                            Some(y)
                        } else {
                            None
                        }
                    },
                )
                .collect();
            let z_km: Vec<_> = nav
                .sv_position()
                .filter_map(
                    |(_epoch, svnn, (_, _, z))| {
                        if svnn == sv {
                            Some(z)
                        } else {
                            None
                        }
                    },
                )
                .collect();
            let trace = build_3d_chart_epoch_label(
                &format!("{:X}", sv),
                Mode::Markers,
                epochs.clone(),
                x_km,
                y_km,
                z_km,
            )
            .visible({
                if sv_index == 0 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
            plot_ctx.add_trace(trace);
        }
    }
    /*
     * add SP3 (x, y, z) position, if available
     */
    if let Some(sp3) = ctx.sp3() {
        for (sv_index, sv) in sp3.sv().enumerate() {
            if !nav_sv.contains(&sv) {
                continue;
            }
            if sv_index == 0 && !pos_plot_created {
                plot_ctx.add_cartesian3d_plot("SV Orbit (broadcast)", "x [km]", "y [km]", "z [km]");
                trace!("broadcast orbit plot");
                pos_plot_created = true;
            }
            let epochs: Vec<_> = sp3
                .sv_position()
                .filter_map(
                    |(epoch, svnn, (_x, _, _))| {
                        if svnn == sv {
                            Some(epoch)
                        } else {
                            None
                        }
                    },
                )
                .collect();
            let x: Vec<f64> = sp3
                .sv_position()
                .filter_map(
                    |(_epoch, svnn, (x, _, _))| {
                        if svnn == sv {
                            Some(x)
                        } else {
                            None
                        }
                    },
                )
                .collect();
            let y: Vec<f64> = sp3
                .sv_position()
                .filter_map(
                    |(_epoch, svnn, (_, y, _))| {
                        if svnn == sv {
                            Some(y)
                        } else {
                            None
                        }
                    },
                )
                .collect();
            let z: Vec<_> = sp3
                .sv_position()
                .filter_map(
                    |(_epoch, svnn, (_, _, z))| {
                        if svnn == sv {
                            Some(z)
                        } else {
                            None
                        }
                    },
                )
                .collect();
            let trace = build_3d_chart_epoch_label(
                &format!("SP3({:X})", sv),
                Mode::LinesMarkers,
                epochs.clone(),
                x,
                y,
                z,
            )
            .visible({
                if sv_index == 0 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
            plot_ctx.add_trace(trace);
        }
    }
}
