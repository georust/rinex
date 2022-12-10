use plotly::{
    ScatterPolar,
    common::{
        Mode,
    },
};
use rinex::prelude::*;

/*
 * Skyplot view
 */
pub fn skyplot(
    rnx: &Rinex,
    nav: &Option<Rinex>,
    ref_pos: Option<(f64, f64, f64)>,
    file: &str,
) {
    /*
    if let Some(nav) = nav {
        /*
         * "advanced" skyplot view,
         * observations were provided
         * color gradient emphasizes the SSI[dB]
         */
        let obs_rec = rnx.record.as_obs()
            .expect("--fp should be Observation RINEX");
        let nav_rec = nav.record.as_nav()
            .expect("--nav should be Navigation RINEX");

        // determine epoch boundaries
        //  this will help emphasize the curves starting and endint points
        let epochs = nav.epochs();
        let e_0 = epochs[0];
        let e_N = epochs[epochs.len()-1];

        // build dataset
        let dataset: HashMap<Sv, HashMap<Epoch, f64>> = HashMap::new();
        for (epoch, classes) in nav_rec {

        }

    } else {*/
    /*
     * "simplified" skyplot view,
     * color gradient emphasizes the epoch/timestamp
     */
    if let Some(r) = rnx.record.as_nav() {
        let mut sat_angles = rnx.navigation_sat_angles(ref_pos);
        for (sv, epochs) in sat_angles {
            let el: Vec<_> = epochs
                .iter()
                .map(|(_, (el,_))| {
                    el.clone()
                }).collect();
            let azi: Vec<_> = epochs
                .iter()
                .map(|(_, (_,azi))| {
                    azi.clone()
                }).collect();
            let trace = ScatterPolar::new(el, azi)
                .mode(Mode::Lines);
            //plot.add_trace(trace);
        }
    }
    //plot.show();
}
