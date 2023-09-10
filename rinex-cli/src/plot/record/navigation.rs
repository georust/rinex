//! Navigation record plotting
use crate::plot::{build_chart_epoch_axis, PlotContext};
use plotly::common::{Mode, Visible};
use rinex::prelude::*;
use rinex_qc::QcContext;

pub fn plot_navigation(ctx: &QcContext, plot_context: &mut PlotContext) {
    /*
     * NB: this does not produce anything to this day,
     *     even when invoked on primary OBS
     *     because we're only plotting the rinex.clock_data() iterator
     */
    if ctx.primary_data().is_navigation_rinex() {
        plot_nav_data(&ctx.primary_data(), plot_context);
    } else if let Some(nav) = &ctx.navigation_data() {
        /* unreached if NAV is the primary data type */
        plot_nav_data(nav, plot_context);
    }
}

fn plot_nav_data(rinex: &Rinex, plot_ctx: &mut PlotContext) {
    let epochs: Vec<_> = rinex.epoch().collect();
    /*
     * Sv Clock and Clock drift visualization
     * one plot (2 Y axes) for both sets
     */
    plot_ctx.add_cartesian2d_2y_plot("Sv Clock Bias", "Clock Bias [s]", "Clock Drift [s/s]");
    for (sv_index, sv) in rinex.sv().enumerate() {
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
            &format!("{}(clk)", sv),
            Mode::LinesMarkers,
            epochs.clone(),
            sv_clock,
        )
        .web_gl_mode(true)
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
            &format!("{}(drift)", sv),
            Mode::LinesMarkers,
            epochs.clone(),
            sv_drift,
        )
        .web_gl_mode(true)
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
    trace!("broadcast ephemeris : sv clock");
    trace!("broadcast ephemeris : sv clock drift");
    /*
     * SV position in sky (broadcast ephemeris)
     * one plot with 2 axes {x, y} ECEF
     */
    plot_ctx.add_cartesian2d_2y_plot(
        "Broadcast Ephemeris",
        "SV Position (x) [km]",
        "SV Position (y) [km]",
    );
    for (sv_index, sv) in rinex.sv().enumerate() {
        let epochs: Vec<_> = rinex
            .sv_position()
            .filter_map(
                |(e, svnn, (_x, _, _))| {
                    if svnn == sv {
                        Some(e)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let pos_x: Vec<_> = rinex
            .sv_position()
            .filter_map(|(_e, svnn, (x, _, _))| {
                if svnn == sv {
                    Some(x / 1000.0) // km: standard units when dealing with Sv
                } else {
                    None
                }
            })
            .collect();

        let trace = build_chart_epoch_axis(
            &format!("{}(X[km])", sv),
            Mode::LinesMarkers,
            epochs.clone(),
            pos_x,
        )
        .web_gl_mode(true)
        .visible({
            if sv_index == 0 {
                // Visualize only 1 vehicle
                Visible::True
            } else {
                Visible::LegendOnly
            }
        });
        plot_ctx.add_trace(trace);

        let pos_y: Vec<_> = rinex
            .sv_position()
            .filter_map(|(_e, svnn, (_, y, _))| {
                if svnn == sv {
                    Some(y / 1000.0) // km: standard units when dealing with Sv
                } else {
                    None
                }
            })
            .collect();

        let trace = build_chart_epoch_axis(
            &format!("{}(Y[km])", sv),
            Mode::LinesMarkers,
            epochs.clone(),
            pos_y,
        )
        .web_gl_mode(true)
        .visible({
            if sv_index == 0 {
                // Visualize only 1 vehicle
                Visible::True
            } else {
                Visible::LegendOnly
            }
        });
        plot_ctx.add_trace(trace);
    }
    /*
     * SV position in sky (broadcast ephemeris): altitude ECEF
     */
    plot_ctx.add_cartesian2d_plot("Broadcast Ephemeris", "SV Altitude [km]");
    for (sv_index, sv) in rinex.sv().enumerate() {
        let epochs: Vec<_> = rinex
            .sv_position()
            .filter_map(
                |(e, svnn, (_x, _, _))| {
                    if svnn == sv {
                        Some(e)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let pos_z: Vec<_> = rinex
            .sv_position()
            .filter_map(|(_e, svnn, (_, _, z))| {
                if svnn == sv {
                    Some(z / 1000.0) // km: standard units when dealing with Sv
                } else {
                    None
                }
            })
            .collect();

        let trace = build_chart_epoch_axis(
            &format!("{}(alt[km])", sv),
            Mode::LinesMarkers,
            epochs.clone(),
            pos_z,
        )
        .web_gl_mode(true)
        .visible({
            if sv_index == 0 {
                // Visualize only 1 vehicle
                Visible::True
            } else {
                Visible::LegendOnly
            }
        });
        plot_ctx.add_trace(trace);
    }
    trace!("broadcast ephemeris : sv 3D positions");
}
