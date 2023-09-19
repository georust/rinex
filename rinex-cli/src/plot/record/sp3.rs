use crate::plot::{build_chart_epoch_axis, PlotContext};
use plotly::common::{Mode, Visible}; //Marker, MarkerSymbol
use rinex::prelude::Epoch;
use rinex_qc::QcContext;

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
    }
    /*
     * Position residual errors : [m]
     * 1 trace (color) per SV
     */
    for (sv_index, sv) in nav.sv().enumerate() {
        if sv_index == 0 {
            plot_ctx.add_cartesian2d_plot(
                "Broadast Ephemeris Residual Position Error (|NAV-SP3|)",
                "Error [m]",
            );
            trace!("|sp3 - broadcast| residual (x, y, z) error");
        }
        let sv_position = nav
            .sv_position()
            .filter_map(|(t, nav_sv, (x_km, y_km, z_km))| {
                if sv == nav_sv {
                    Some((t, (x_km, y_km, z_km)))
                } else {
                    None
                }
            });

        let mut epochs: Vec<Epoch> = Vec::new();
        let mut residuals: Vec<f64> = Vec::new();

        for (t, (sv_x, sv_y, sv_z)) in sv_position {
            if let Some((_, _, (sp3_x, sp3_y, sp3_z))) = sp3
                .sv_position()
                .find(|(e_sp3, sv_sp3, (_, _, _))| *e_sp3 == t && *sv_sp3 == sv)
            {
                /* no need to interpolate => use right away */
                epochs.push(t);
                let err = ((sp3_x / 1000.0 - sv_x / 1000.0).powi(2)
                    + (sp3_y / 1000.0 - sv_y / 1000.0).powi(2)
                    + (sp3_z / 1000.0 - sv_z / 1000.0).powi(2))
                .sqrt();
                residuals.push(err);
            } else {
                /* needs interpolation */
                if let Some((sp3_x, sp3_y, sp3_z)) = sp3.sv_position_interpolate(sv, t, 11) {
                    epochs.push(t);
                    let err = ((sp3_x / 1000.0 - sv_x / 1000.0).powi(2)
                        + (sp3_y / 1000.0 - sv_y / 1000.0).powi(2)
                        + (sp3_z / 1000.0 - sv_z / 1000.0).powi(2))
                    .sqrt();
                    residuals.push(err);
                }
            }
        }
        let trace =
            build_chart_epoch_axis(&format!("|{}_err|", sv), Mode::Markers, epochs, residuals)
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
