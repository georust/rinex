use plotly::{
    ScatterPolar,
    common::{
        Mode,
    },
};
use super::Context;
use rinex::prelude::*;

/*
 * Skyplot view
 */
pub fn skyplot(
    ctx: &mut Context,
    rnx: &Rinex,
    nav: &Option<Rinex>,
    ref_pos: Option<(f64, f64, f64)>,
) {
    ctx.add_polar2d_plot("Skyplot");
    if let Some(nav) = nav {
        /*
         * "advanced" skyplot view,
         * observations were provided
         * color gradient emphasizes the SSI[dB]
         */
        if !nav.is_navigation_rinex() {
            println!("--nav should be Navigation Data!");
            return;
        }
        
        let sat_angles = nav.navigation_sat_angles(ref_pos);
        for (sv, epochs) in sat_angles {
            let el: Vec<f64> = epochs
                .iter()
                .map(|(_, (el,_))| {
                    el * 360.0 / std::f64::consts::PI  
                }).collect();
            let azi: Vec<f64> = epochs
                .iter()
                .map(|(_, (_,azi))| {
                    azi * 360.0 / std::f64::consts::PI  
                }).collect();
            let trace = ScatterPolar::new(el, azi)
                .mode(Mode::LinesMarkers)
                .name(sv.to_string());
            ctx.add_trace(trace);
        }

    } else {
        /*
         * "simplified" skyplot view,
         * color gradient emphasizes the epoch/timestamp
         */
        let sat_angles = rnx.navigation_sat_angles(ref_pos);
        for (sv, epochs) in sat_angles {
            let el: Vec<f64> = epochs
                .iter()
                .map(|(_, (el,_))| {
                    el * 360.0 / std::f64::consts::PI  
                }).collect();
            let azi: Vec<f64> = epochs
                .iter()
                .map(|(_, (_,azi))| {
                    azi * 360.0 / std::f64::consts::PI  
                }).collect();
            let trace = ScatterPolar::new(el, azi)
                .mode(Mode::LinesMarkers)
                .name(sv.to_string());
            ctx.add_trace(trace);
        }
    }
}
