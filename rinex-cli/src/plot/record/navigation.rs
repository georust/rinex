use crate::plot::{build_3d_chart_epoch_label, build_chart_epoch_axis, PlotContext};
use plotly::common::{Mode, Visible};
use rinex::navigation::Ephemeris;
use rinex::prelude::*;

pub fn plot_navigation(ctx: &RnxContext, plot_ctx: &mut PlotContext) {
    let mut clock_plot_created = false;
    if let Some(nav) = ctx.nav_data() {
        /*
         * Plot SV Clock Offset/Drift
         * one plot (2 Y axes) for both Clock biases
         * and clock drift
         */
        for (sv_index, sv) in nav.sv().enumerate() {
            if sv_index == 0 {
                plot_ctx.add_timedomain_2y_plot(
                    "SV Clock Bias",
                    "Clock Bias [s]",
                    "Clock Drift [s/s]",
                );
                trace!("sv clock plot");
                clock_plot_created = true;
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
                &format!("{:X}(clk)", sv),
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
    }
    /*
     * Plot similar SP3 data (if any)
     */
    if let Some(sp3) = ctx.sp3_data() {
        for (sv_index, sv) in sp3.sv().enumerate() {
            if sv_index == 0 && !clock_plot_created {
                plot_ctx.add_timedomain_2y_plot(
                    "SV Clock Bias",
                    "Clock Bias [s]",
                    "Clock Drift [s/s]",
                );
                trace!("sv clock plot");
                clock_plot_created = true;
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
            let trace =
                build_chart_epoch_axis(&format!("{:X}(sp3_clk)", sv), Mode::Markers, epochs, data)
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
     * Plot Broadcast Orbit (x, y, z)
     */
    let mut pos_plot_created = false;
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
                plot_ctx.add_cartesian3d_plot(
                    "SV Orbit (broadcast)",
                    "x [km]",
                    "y [km]",
                    "z [km]",
                );
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
    /*
     * Plot BRDC Clock correction
     */
    if let Some(navdata) = ctx.nav_data() {
        if let Some(obsdata) = ctx.obs_data() {
            for (sv_index, sv) in obsdata.sv().enumerate() {
                if sv_index == 0 {
                    plot_ctx.add_timedomain_plot("SV Clock Correction", "Correction [s]");
                    trace!("sv clock correction plot");
                }
                let epochs: Vec<_> = obsdata
                    .observation()
                    .filter_map(|((t, flag), (_, vehicles))| {
                        if flag.is_ok() && vehicles.contains_key(&sv) {
                            Some(*t)
                        } else {
                            None
                        }
                    })
                    .collect();
                let clock_corr: Vec<_> = obsdata
                    .observation()
                    .filter_map(|((t, flag), (_, _vehicles))| {
                        if flag.is_ok() {
                            let (toe, sv_eph) = navdata.sv_ephemeris(sv, *t)?;
                            /*
                             * TODO prefer SP3 (if any)
                             */
                            let clock_state = sv_eph.sv_clock();
                            let clock_corr = Ephemeris::sv_clock_corr(sv, clock_state, *t, toe);
                            Some(clock_corr.to_seconds())
                        } else {
                            None
                        }
                    })
                    .collect();

                let trace =
                    build_chart_epoch_axis(&format!("{}", sv), Mode::Markers, epochs, clock_corr)
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
    }
}
