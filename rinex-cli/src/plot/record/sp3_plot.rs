use crate::plot::{build_3d_chart_epoch_label, PlotContext};
use plotly::common::{Mode, Visible}; //Marker, MarkerSymbol
use rinex::prelude::Epoch;
use rinex::prelude::RnxContext;

/*
 * Advanced NAV feature
 * compares residual error between broadcast ephemeris
 * and SP3 high precision orbits
 */
pub fn plot_residual_ephemeris(ctx: &RnxContext, plot_ctx: &mut PlotContext) {
    let sp3 = ctx
        .sp3_data() // cannot fail at this point
        .unwrap();
    let nav = ctx
        .nav_data() // cannot fail at this point
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
            plot_ctx.add_cartesian3d_plot(
                "Broadast / SP3 Residual Position Error",
                "dx [m]",
                "dy [m]",
                "dz [m]",
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
        let mut residuals: Vec<(f64, f64, f64)> = Vec::new();

        for (t, (sv_x, sv_y, sv_z)) in sv_position {
            if let Some((_, _, (sp3_x, sp3_y, sp3_z))) = sp3
                .sv_position()
                .find(|(e_sp3, sv_sp3, (_, _, _))| *e_sp3 == t && *sv_sp3 == sv)
            {
                /* no need to interpolate => use right away */
                epochs.push(t);
                residuals.push((
                    (sv_x - sp3_x) / 1.0E3,
                    (sv_y - sp3_y) / 1.0E3,
                    (sv_z - sp3_z) / 1.0E3,
                ));
            } else {
                /* needs interpolation */
                if let Some((sp3_x, sp3_y, sp3_z)) = sp3.sv_position_interpolate(sv, t, 11) {
                    epochs.push(t);
                    residuals.push((
                        (sv_x - sp3_x) / 1.0E3,
                        (sv_y - sp3_y) / 1.0E3,
                        (sv_z - sp3_z) / 1.0E3,
                    ));
                }
            }
        }
        let trace = build_3d_chart_epoch_label(
            &format!("{:X}", sv),
            Mode::Markers,
            epochs,
            residuals.iter().map(|(x, _, _)| *x).collect::<Vec<f64>>(),
            residuals.iter().map(|(_, y, _)| *y).collect::<Vec<f64>>(),
            residuals.iter().map(|(_, _, z)| *z).collect::<Vec<f64>>(),
        )
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
