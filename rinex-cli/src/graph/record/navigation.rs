use crate::graph::{build_3d_chart_epoch_label, build_chart_epoch_axis, PlotContext};
use plotly::common::{Mode, Visible};
use rinex::navigation::Ephemeris;
use rinex::prelude::*;

use sp3::SP3;

pub fn plot_sv_nav_clock(ctx: &RnxContext, plot_ctx: &mut PlotContext) {
    if let Some(nav) = ctx.nav_data() {
        let clk = ctx.clk_data();
        let sp3 = ctx.sp3_data();
        plot_sv_clock_attitude(nav, clk, sp3, plot_ctx);
        if let Some(obs) = ctx.obs_data() {
            plot_sv_clock_corrections(obs, nav, clk, sp3, plot_ctx);
        } else {
            info!("adding OBS RINEX will provide clock corrections graphs");
        }
    } else {
        warn!("missing navigation (orbits) data");
    }
}

/*
 * Plot SV Clock Offset/Drift
 * one plot (2 Y axes) for both Clock biases
 * and clock drift
 */
fn plot_sv_clock_attitude(
    nav: &Rinex,
    clk: Option<&Rinex>,
    sp3: Option<&SP3>,
    plot_ctx: &mut PlotContext,
) {
    let mut plot_created = false;
    for (sv_index, sv) in nav.sv().enumerate() {
        if sv_index == 0 {
            plot_ctx.add_timedomain_2y_plot("SV Clock Bias", "Clock Bias [s]", "Clock Drift [s/s]");
            trace!("sv clock plot");
            plot_created = true;
        }
        let sv_epochs: Vec<_> = nav
            .sv_clock()
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
        let sv_clock: Vec<_> = nav
            .sv_clock()
            .filter_map(
                |(_epoch, svnn, (clk, _, _))| {
                    if svnn == sv {
                        Some(clk)
                    } else {
                        None
                    }
                },
            )
            .collect();

        let sv_drift: Vec<_> = nav
            .sv_clock()
            .filter_map(
                |(_epoch, svnn, (_, drift, _))| {
                    if svnn == sv {
                        Some(drift)
                    } else {
                        None
                    }
                },
            )
            .collect();

        let trace = build_chart_epoch_axis(
            &format!("{:X}(offset)", sv),
            Mode::LinesMarkers,
            sv_epochs.clone(),
            sv_clock,
        )
        .visible({
            if sv_index == 0 {
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
            if sv_index == 0 {
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

    /*
     * Plot similar SP3 data (if any)
     */
    if let Some(sp3) = sp3 {
        for (sv_index, sv) in sp3.sv().enumerate() {
            if sv_index == 0 && !plot_created {
                plot_ctx.add_timedomain_2y_plot(
                    "SV Clock Bias",
                    "Clock Bias [s]",
                    "Clock Drift [s/s]",
                );
                trace!("sv clock plot");
                plot_created = true;
            }

            let epochs: Vec<_> = sp3
                .sv_clock()
                .filter_map(
                    |(epoch, svnn, _clk)| {
                        if svnn == sv {
                            Some(epoch)
                        } else {
                            None
                        }
                    },
                )
                .collect();
            let data: Vec<_> = sp3
                .sv_clock()
                .filter_map(
                    |(_, svnn, clk)| {
                        if svnn == sv {
                            Some(clk * 1.0E-6)
                        } else {
                            None
                        }
                    },
                )
                .collect();
            let trace = build_chart_epoch_axis(
                &format!("{:X}(offset;sp3)", sv),
                Mode::Markers,
                epochs,
                data,
            )
            .visible({
                if sv_index == 0 {
                    // Clock data differs too much: plot only one to begin with
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
            plot_ctx.add_trace(trace);
        }
    }

    /*
     * Plot similar CLK data (if any)
     */
    if let Some(clk) = clk {
        for (sv_index, sv) in clk.sv().enumerate() {
            if sv_index == 0 && !plot_created {
                plot_ctx.add_timedomain_2y_plot(
                    "SV Clock Bias",
                    "Clock Bias [s]",
                    "Clock Drift [s/s]",
                );
                trace!("sv clock plot");
                plot_created = true;
            }

            let clk_bias: Vec<_> = clk
                .sv_embedded_clock()
                .filter_map(|(epoch, svnn, _, profile)| {
                    if svnn == sv {
                        Some((epoch, profile.bias))
                    } else {
                        None
                    }
                })
                .collect();

            let clk_bias_epochs = clk_bias.iter().map(|(e, _)| *e).collect();
            let clk_biases = clk_bias.iter().map(|(_, b)| *b).collect();

            let trace = build_chart_epoch_axis(
                &format!("{:X}(offset;clk)", sv),
                Mode::Markers,
                clk_bias_epochs,
                clk_biases,
            )
            .visible({
                if sv_index == 0 {
                    // Clock data differs too much: plot only one to begin with
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
            plot_ctx.add_trace(trace);

            let clk_drift: Vec<_> = clk
                .sv_embedded_clock()
                .filter_map(|(epoch, svnn, _, profile)| {
                    if svnn == sv {
                        if let Some(drift) = profile.drift {
                            Some((epoch, drift))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            let clk_drift_epochs = clk_drift.iter().map(|(e, _)| *e).collect();
            let clk_drifts = clk_drift.iter().map(|(_, d)| *d).collect();

            let trace = build_chart_epoch_axis(
                &format!("{:X}(drift;clk)", sv),
                Mode::LinesMarkers,
                clk_drift_epochs,
                clk_drifts,
            )
            .y_axis("y2")
            .visible({
                if sv_index == 0 {
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
 * Plot SV clock corrections
 */
fn plot_sv_clock_corrections(
    obs: &Rinex,
    nav: &Rinex,
    clk: Option<&Rinex>,
    sp3: Option<&SP3>,
    plot_ctx: &mut PlotContext,
) {
    for (sv_index, sv) in obs.sv().enumerate() {
        if sv_index == 0 {
            plot_ctx.add_timedomain_plot("SV Clock Correction", "Correction [s]");
            trace!("brdc clock correction plot");
        }
        let epochs: Vec<_> = obs
            .observation()
            .filter_map(|((t, flag), (_, vehicles))| {
                if flag.is_ok() && vehicles.contains_key(&sv) {
                    Some(*t)
                } else {
                    None
                }
            })
            .collect();
        let clock_corr: Vec<_> = obs
            .observation()
            .filter_map(|((t, flag), (_, _vehicles))| {
                if flag.is_ok() {
                    let (toe, sv_eph) = nav.sv_ephemeris(sv, *t)?;
                    let clock_state =
                    // 1. prefer CLK as source
                    // 2. then SP3
                    // 3. and then BRDC as last option
                    if let Some(clk) = clk {
                        let (_, profile) = clk
                            .sv_embedded_clock_interpolate(*t, sv)?;
                        (profile.bias, 0.0_f64, 0.0_f64) //TODO
                    } else if let Some(sp3) = sp3 {
                        sp3
                            .sv_clock()
                            .filter_map(|(sp3_t, sp3_sv, bias)| {
                                if sp3_t == *t && sp3_sv == sv {
                                    Some((bias * 1E-6, 0.0_f64, 0.0_f64))
                                } else {
                                    None
                                }
                            })
                            .reduce(|k, _| k)?
                    } else {
                        sv_eph.sv_clock()
                    };
                    let clock_corr = Ephemeris::sv_clock_corr(sv, clock_state, *t, toe);
                    Some(clock_corr.to_seconds())
                } else {
                    None
                }
            })
            .collect();

        let trace = build_chart_epoch_axis(&format!("{}", sv), Mode::Markers, epochs, clock_corr)
            .visible({
                if sv_index < 3 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
        plot_ctx.add_trace(trace);
    }
}

pub fn plot_sv_nav_orbits(ctx: &RnxContext, plot_ctx: &mut PlotContext) {
    let mut pos_plot_created = false;
    /*
     * Plot Broadcast Orbit (x, y, z)
     */
    if let Some(nav) = ctx.nav_data() {
        for (sv_index, sv) in nav.sv().enumerate() {
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
    if let Some(sp3) = ctx.sp3_data() {
        for (sv_index, sv) in sp3.sv().enumerate() {
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
