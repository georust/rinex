//! Navigation record plotting
use crate::plot::{build_chart_epoch_axis, PlotContext};
use plotly::common::{Mode, Visible};
use rinex::prelude::RnxContext;
use rinex::prelude::*;
use sp3::SP3;

pub fn plot_navigation(ctx: &RnxContext, plot_context: &mut PlotContext) {
    if ctx.primary_data().is_navigation_rinex() {
        plot_nav_data(ctx.primary_data(), ctx.sp3_data(), plot_context);
    } else if let Some(nav) = &ctx.navigation_data() {
        plot_nav_data(nav, ctx.sp3_data(), plot_context);
    }
}

fn plot_nav_data(rinex: &Rinex, sp3: Option<&SP3>, plot_ctx: &mut PlotContext) {
    /*
     * Plot SV Clock Offset/Drift
     * one plot (2 Y axes) for both Clock biases
     * and clock drift
     */
    for (sv_index, sv) in rinex.sv().enumerate() {
        if sv_index == 0 {
            plot_ctx.add_cartesian2d_2y_plot(
                "SV Clock Bias",
                "Clock Bias [s]",
                "Clock Drift [s/s]",
            );
            trace!("sv clock plot");
        }
        let sv_epochs: Vec<_> = rinex
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
        let sv_clock: Vec<_> = rinex
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

        let sv_drift: Vec<_> = rinex
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
        /*
         * Plot SP3 similar data (if available)
         * useful for comparison
         */
        if let Some(sp3) = sp3 {
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
     * Plot Broadcast Orbits
     * X/Y(ECEF) on the sample plot
     * Z (altitude) has its own plot
     */
    for (sv_index, sv) in rinex.sv().enumerate() {
        if sv_index == 0 {
            plot_ctx.add_cartesian2d_2y_plot("SV Orbit", "Position (x) [km]", "Position (y) [km]");
            trace!("sv orbit plot");
        }
        let epochs: Vec<_> = rinex
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

        let x_km: Vec<_> = rinex
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
        let trace =
            build_chart_epoch_axis(&format!("{:X}(x)", sv), Mode::Markers, epochs.clone(), x_km)
                .visible({
                    if sv_index == 0 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                });
        plot_ctx.add_trace(trace);

        let y_km: Vec<_> = rinex
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

        let trace =
            build_chart_epoch_axis(&format!("{:X}(y)", sv), Mode::Markers, epochs.clone(), y_km)
                .y_axis("y2")
                .visible({
                    if sv_index == 0 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                });
        plot_ctx.add_trace(trace);
        /*
         * add SP3 (x, y) positions, if available
         */
        if let Some(sp3) = sp3 {
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
            let trace = build_chart_epoch_axis(
                &format!("{:X}(sp3_x)", sv),
                Mode::LinesMarkers,
                epochs.clone(),
                x,
            )
            .visible({
                if sv_index == 0 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
            plot_ctx.add_trace(trace);
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
            let trace = build_chart_epoch_axis(
                &format!("{:X}(sp3_y)", sv),
                Mode::LinesMarkers,
                epochs.clone(),
                y,
            )
            .y_axis("y2")
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

    for (sv_index, sv) in rinex.sv().enumerate() {
        if sv_index == 0 {
            plot_ctx.add_cartesian2d_plot("SV Altitude", "Altitude [km]");
            trace!("sv altitude plot");
        }
        let epochs: Vec<_> = rinex
            .sv_position()
            .filter_map(
                |(epoch, svnn, (_, _, _z))| {
                    if svnn == sv {
                        Some(epoch)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let z_km: Vec<_> = rinex
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
        let trace = build_chart_epoch_axis(&format!("{:X}(z)", sv), Mode::Markers, epochs, z_km)
            .visible({
                if sv_index == 0 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
        plot_ctx.add_trace(trace);
        /*
         * add SP3 (z) positions, if available
         */
        if let Some(sp3) = sp3 {
            let epochs: Vec<_> = sp3
                .sv_position()
                .filter_map(
                    |(epoch, svnn, (_, _, _z))| {
                        if svnn == sv {
                            Some(epoch)
                        } else {
                            None
                        }
                    },
                )
                .collect();
            let z_km: Vec<_> = sp3
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
            let trace = build_chart_epoch_axis(
                &format!("{:X}(sp3_z)", sv),
                Mode::LinesMarkers,
                epochs,
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
}
