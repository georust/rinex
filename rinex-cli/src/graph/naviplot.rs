use crate::graph::PlotContext;
// use plotly::{
//     common::{Mode, Visible},
//     ScatterPolar,
// };
// use rinex::prelude::Epoch;
use rinex_qc::prelude::DataContext;

/*
 * NAVI plot is an advanced 3D view
 * which is basically the skyplot view with observation signals
 * and ionospheric conditions taken into account
 */
pub fn naviplot(_ctx: &DataContext, plot_context: &mut PlotContext) {
    plot_context.add_cartesian3d_plot("NAVI Plot", "x", "y", "z");

    // grab NAV context
    // let nav_rnx = match &ctx.navigation_data() {
    //     Some(nav) => nav,
    //     _ => ctx.primary_data(),
    // };
}
