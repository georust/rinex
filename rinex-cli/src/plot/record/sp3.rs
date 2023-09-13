use crate::plot::{build_chart_epoch_axis, PlotContext};
use plotly::color::NamedColor;
use plotly::common::{Marker, MarkerSymbol, Mode, Visible};
use rinex::prelude::Epoch;
use rinex_qc::QcContext;
/*
 * Plots High Precision Orbit and Clock data
 * provided in the form of an SP3 file
 */
pub fn plot_sp3(ctx: &QcContext, plot_context: &mut PlotContext) {
    let sp3 = ctx.sp3_data().unwrap(); // cannot fail at this point
                                       /*
                                        * Plot SV Position
                                        * 1 color per SV
                                        */
    for (sv_index, sv) in sp3.sv().enumerate() {
        if sv_index == 0 {
            plot_context.add_cartesian2d_2y_plot(
                "High Precision Orbit (SP3)",
                "SV Position (x) [km]",
                "SV Position (y) [km]",
            );
            trace!("sp3 orbits visualization");
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
        let trace = build_chart_epoch_axis(&format!("{}(x)", sv), Mode::Markers, epochs.clone(), x)
            .web_gl_mode(true)
            .visible({
                if sv_index == 0 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
        plot_context.add_trace(trace);
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
        let trace = build_chart_epoch_axis(&format!("{}(y)", sv), Mode::Markers, epochs.clone(), y)
            .web_gl_mode(true)
            .y_axis("y2")
            .visible({
                if sv_index == 0 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
        plot_context.add_trace(trace);
    }

    for (sv_index, sv) in sp3.sv().enumerate() {
        if sv_index == 0 {
            plot_context.add_cartesian2d_plot("High Precision Altitude (SP3)", "SV Altitude (km)");
            trace!("sp3 altitude visualization");
        }
        let data_x: Vec<Epoch> = sp3
            .sv_position()
            .filter_map(
                |(epoch, svnn, _)| {
                    if svnn == sv {
                        Some(epoch)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let data_y: Vec<f64> = sp3
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
        let trace = build_chart_epoch_axis(&format!("{}(z)", sv), Mode::Markers, data_x, data_y)
            .web_gl_mode(true)
            .visible({
                if sv_index < 4 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
        plot_context.add_trace(trace);
    }
    trace!("sp3 orbits visualization");
    /*
     * Plot SV Clock data prediction
     */
    plot_context.add_cartesian2d_plot("High Precision Clock (SP3)", "SV Clock Bias [us]");
    for (sv_index, sv) in sp3.sv().enumerate() {
        let data_x: Vec<Epoch> = sp3
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
        let data_y: Vec<f64> = sp3
            .sv_clock()
            .filter_map(
                |(_epoch, svnn, clk)| {
                    if svnn == sv {
                        Some(clk)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let trace = build_chart_epoch_axis(&format!("{}", sv), Mode::Markers, data_x, data_y)
            .web_gl_mode(true)
            .visible({
                if sv_index == 0 {
                    // Clock data differs too much: plot only one to begin with
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
        plot_context.add_trace(trace);
    }
    trace!("sp3 clock data visualization");
}

/*
 * Advanced NAV feature
 * compares residual error between broadcast ephemeris
 * and SP3 high precision orbits
 */
pub fn plot_residual_ephemeris(ctx: &QcContext, plot_ctx: &mut PlotContext) {
    let sp3 = ctx
        .sp3_data() // cannot fail at this point
        .unwrap();
    let nav = ctx
        .navigation_data() // cannot fail at this point
        .unwrap();
    /*
     * we need at least a small common time frame,
     * otherwise analysis is not feasible
     */
    let mut feasible = false;
    if let (Some(first_sp3), Some(last_sp3)) = (sp3.first_epoch(), sp3.last_epoch()) {
        if let (Some(first_nav), Some(last_nav)) = (nav.first_epoch(), nav.last_epoch()) {
            feasible |= (first_nav > first_sp3) && (first_nav < last_sp3);
            feasible |= (last_nav > first_sp3) && (last_nav < last_sp3);
        }
    }

    if !feasible {
        warn!("|sp3-nav| residual analysis not feasible due to non common time frame");
        return;
    } else {
        trace!("|sp3-nav| residual analysis..");
    }
    plot_ctx.add_cartesian2d_plot(
        "Broadast Ephemeris Residual Errors (|NAV-SP3|)",
        "Residual error [m]",
    );
    /*
     * 1 trace/color per SV
     */
    for (index, sv) in nav.sv().enumerate() {
        let sv_position = nav.sv_position().filter_map(|(t, (nav_sv, x, y, z))| {
            if sv == nav_sv {
                Some((t, (x, y, z)))
            } else {
                None
            }
        });

        let mut epochs: Vec<Epoch> = Vec::new();
        let mut residuals: Vec<f64> = Vec::new();
        let mut interp_epochs: Vec<Epoch> = Vec::new();
        let mut interp_residuals: Vec<f64> = Vec::new();
        for (t, (x, y, z)) in sv_position {
            if let Some((_, _, (x_km, y_km, z_km))) = sp3
                .sv_position()
                .find(|(e_sp3, sv_sp3, (_, _, _))| *e_sp3 == t && *sv_sp3 == sv)
            {
                epochs.push(t);
                let err = ((x_km - x).powi(2) + (y_km - y).powi(2) + (z_km - z).powi(2)).sqrt();
                residuals.push(err);
            } else {
                /* needs interpolation */
                if let Some((x_km, y_km, z_km)) = sp3.sv_position_interpolate(sv, t, 11) {
                    interp_epochs.push(t);
                    let err = ((x_km - x).powi(2) + (y_km - y).powi(2) + (z_km - z).powi(2)).sqrt();
                    residuals.push(err);
                }
            }
        }
        let trace = build_chart_epoch_axis(&format!("{}", sv), Mode::Markers, epochs, residuals)
            .web_gl_mode(true)
            .visible({
                if index == 0 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
        plot_ctx.add_trace(trace);

        let trace = build_chart_epoch_axis(
            &format!("interp({})", sv),
            Mode::Markers,
            interp_epochs,
            interp_residuals,
        )
        .web_gl_mode(true)
        .marker(
            Marker::new()
                .symbol(MarkerSymbol::TriangleDown)
                .color(NamedColor::Red),
        )
        .visible({
            if index == 0 {
                Visible::True
            } else {
                Visible::LegendOnly
            }
        });
        plot_ctx.add_trace(trace);
    }
    trace!("sp3/ephemeris residual error");
}
