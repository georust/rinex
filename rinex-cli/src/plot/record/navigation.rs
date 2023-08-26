//! Navigation record plotting
use crate::plot::{build_chart_epoch_axis, PlotContext};
use plotly::common::{Mode, Visible};
use rinex::prelude::*;
use rinex::quality::QcContext;

pub fn plot_navigation(ctx: &QcContext, plot_context: &mut PlotContext) {
    /*
     * one plot (2 Y axes) for both Clock biases
     * and clock drift
     */
    plot_context.add_cartesian2d_2y_plot("Sv Clock Bias", "Clock Bias [s]", "Clock Drift [s/s]");

    /*
     * NB: this does not produce anything to this day,
     *     even when invoked on primary OBS
     *     because we're only plotting the rinex.clock_data() iterator
     */
    plot_nav_data(&ctx.primary_data(), plot_context);
    if let Some(nav) = &ctx.navigation_data() {
        plot_nav_data(nav, plot_context);
    }
}

fn plot_nav_data(rinex: &Rinex, plot_ctx: &mut PlotContext) {
    let epochs: Vec<_> = rinex.epoch().collect();

    for (sv_index, sv) in rinex.sv().enumerate() {
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
    trace!("navigation data plot");
}
