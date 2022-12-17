use super::PlotContext;
use crate::Context;
use plotly::{
    common::{Mode, Visible},
    ScatterPolar,
};
/*
 * Skyplot view
 */
pub fn skyplot(ctx: &Context, plot_ctx: &mut PlotContext) {
    plot_ctx.add_polar2d_plot("Skyplot");
    if let Some(ref nav) = ctx.nav_rinex {
        /*
         * "advanced" skyplot view,
         * observations were provided
         * color gradient emphasizes the SSI[dB]
         */
        if !nav.is_navigation_rinex() {
            println!("--nav should be Navigation Data!");
            return;
        }

        let sat_angles = ctx.primary_rinex.navigation_sat_angles(ctx.ground_position);
        for (index, (sv, epochs)) in sat_angles.iter().enumerate() {
            let elev: Vec<f64> = epochs.iter().map(|(epoch, (elev, _))| *elev).collect();
            let azim: Vec<f64> = epochs.iter().map(|(_, (_, azim))| *azim).collect();
            let trace = ScatterPolar::new(elev, azim)
                .mode(Mode::LinesMarkers)
                .visible({
                    if index < 4 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                })
                .connect_gaps(false)
                .name(sv.to_string());
            plot_ctx.add_trace(trace);
        }
    } else {
        /*
         * "simplified" skyplot view,
         * color gradient emphasizes the epoch/timestamp
         */
        let sat_angles = ctx.primary_rinex.navigation_sat_angles(ctx.ground_position);
        for (index, (sv, epochs)) in sat_angles.iter().enumerate() {
            let elev: Vec<f64> = epochs.iter().map(|(_, (elev, _))| *elev).collect();
            let azim: Vec<f64> = epochs.iter().map(|(_, (_, azim))| *azim).collect();
            let trace = ScatterPolar::new(elev, azim)
                .mode(Mode::LinesMarkers)
                .web_gl_mode(true)
                .visible({
                    if index < 4 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                })
                .connect_gaps(false)
                .name(sv.to_string());
            plot_ctx.add_trace(trace);
        }
    }
}
