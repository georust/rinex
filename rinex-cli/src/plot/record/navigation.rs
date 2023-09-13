//! Navigation record plotting
use crate::plot::{build_chart_epoch_axis, PlotContext};
use plotly::common::{Mode, Visible};
use rinex::prelude::*;
use rinex_qc::QcContext;

pub fn plot_navigation(ctx: &QcContext, plot_context: &mut PlotContext) {
    if ctx.primary_data().is_navigation_rinex() {
        plot_nav_data(&ctx.primary_data(), plot_context);
    } else if let Some(nav) = &ctx.navigation_data() {
        plot_nav_data(nav, plot_context);
    }
}

fn plot_nav_data(rinex: &Rinex, plot_ctx: &mut PlotContext) {
    let epochs: Vec<_> = rinex.epoch().collect();
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
            trace!("sv clock data visualization");
        }
        let sv_clock: Vec<_> = rinex
            .sv_clock()
            .filter_map(
                |(_epoch, (svnn, (clk, _, _)))| {
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
                |(_epoch, (svnn, (_, drift, _)))| {
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
    /*
     * Plot Broadcast Orbits
     * X/Y(ECEF) on the sample plot
     * Z (altitude) has its own plot
     */
    for (sv_index, sv) in rinex.sv().enumerate() {
        if sv_index == 0 {
            plot_ctx.add_cartesian2d_2y_plot("SV Orbit", "Position (x) [km]", "Position (y) [km]");
            trace!("sv orbits visualization");
        }
        let x_km: Vec<_> = rinex
            .sv_position()
            .filter_map(
                |(_epoch, (svnn, x, _, _))| {
                    if svnn == sv {
                        Some(x)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let trace = build_chart_epoch_axis(
            &format!("{}(x)", sv),
            Mode::LinesMarkers,
            epochs.clone(),
            x_km,
        )
        .web_gl_mode(true)
        .visible({
            if sv_index < 4 {
                Visible::True
            } else {
                Visible::LegendOnly
            }
        });
        plot_ctx.add_trace(trace);

        let y_km: Vec<_> = rinex
            .sv_position()
            .filter_map(
                |(_epoch, (svnn, _, y, _))| {
                    if svnn == sv {
                        Some(y)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let trace = build_chart_epoch_axis(
            &format!("{}(y)", sv),
            Mode::LinesMarkers,
            epochs.clone(),
            y_km,
        )
        .web_gl_mode(true)
        .y_axis("y2")
        .visible({
            if sv_index < 4 {
                Visible::True
            } else {
                Visible::LegendOnly
            }
        });
        plot_ctx.add_trace(trace);
    }

    for (sv_index, sv) in rinex.sv().enumerate() {
        if sv_index == 0 {
            plot_ctx.add_cartesian2d_plot("SV Altitude", "Altitude [km]");
            trace!("sv altitude visualization");
        }
        let epochs: Vec<_> = rinex
            .sv_position()
            .filter_map(
                |(epoch, (svnn, _, _, _z))| {
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
                |(_epoch, (svnn, _, _, z))| {
                    if svnn == sv {
                        Some(z)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let trace = build_chart_epoch_axis(&format!("{}(z)", sv), Mode::LinesMarkers, epochs, z_km)
            .web_gl_mode(true)
            .visible({
                if sv_index < 4 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
        plot_ctx.add_trace(trace);
    }
}
