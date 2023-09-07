use crate::plot::PlotContext;
use plotly::{
    common::{Mode, Visible},
    ScatterPolar,
};
use rinex::prelude::Epoch;
use rinex_qc::QcContext;

/*
 * Skyplot view
 */
pub fn skyplot(ctx: &QcContext, plot_context: &mut PlotContext) {
    plot_context.add_polar2d_plot("Skyplot");

    // grab NAV context
    let nav_rnx = match &ctx.navigation_data() {
        Some(nav) => nav,
        _ => ctx.primary_data(),
    };

    for (svnn_index, svnn) in nav_rnx.sv().enumerate() {
        // per sv
        // grab related elevation data
        // Rho   = degrees(elev)
        // Theta = degrees(azim)
        let data: Vec<(Epoch, f64, f64)> = nav_rnx
            .sv_elevation_azimuth(ctx.ground_position())
            .filter_map(|(epoch, (sv, (elev, azi)))| {
                if sv == svnn {
                    let rho = elev;
                    let theta = azi;
                    Some((epoch, rho, theta))
                } else {
                    None
                }
            })
            .collect();

        let rho: Vec<f64> = data.iter().map(|(_e, rho, _theta)| *rho).collect();
        let theta: Vec<f64> = data.iter().map(|(_e, _rho, theta)| *theta).collect();

        //TODO: color gradient to emphasize day course
        let trace = ScatterPolar::new(theta, rho)
            .mode(Mode::LinesMarkers)
            .web_gl_mode(true)
            .visible({
                /*
                 * Plot only first few dataset,
                 * to improve performance when opening plots
                 */
                if svnn_index < 4 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            })
            .connect_gaps(false)
            .name(svnn.to_string());
        plot_context.add_trace(trace);
    }
}
