use crate::plot::PlotContext;
use plotly::{
    common::{Mode, Visible},
    ScatterPolar,
};
use rinex::prelude::{Epoch, GroundPosition, Rinex};

/*
 * Skyplot view
 */
pub fn skyplot(rnx: &Rinex, ref_position: GroundPosition, plot_context: &mut PlotContext) {
    plot_context.add_polar2d_plot("Skyplot");

    for (svnn_index, svnn) in rnx.sv().enumerate() {
        // per sv
        // grab related elevation data
        // Rho   = degrees(elev)
        // Theta = degrees(azim)
        let data: Vec<(Epoch, f64, f64)> = rnx
            .sv_elevation_azimuth(Some(ref_position))
            .filter_map(|(epoch, sv, (elev, azi))| {
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
            .name(format!("{:X}", svnn));
        plot_context.add_trace(trace);
    }
}
