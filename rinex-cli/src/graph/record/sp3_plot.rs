use crate::graph::{build_chart_epoch_axis, PlotContext};
use plotly::common::{Mode, Visible}; //Marker, MarkerSymbol
use rinex::prelude::{Epoch, SV};
use rinex_qc::prelude::QcContext;
use std::collections::HashMap;

/*
 * Advanced NAV feature
 * compares residual error between broadcast ephemeris
 * and SP3 high precision orbits
 */
pub fn plot_residual_ephemeris(ctx: &QcContext, plot_ctx: &mut PlotContext) {
    let sp3 = ctx
        .sp3() // cannot fail at this point
        .unwrap();
    let nav = ctx
        .brdc_navigation() // cannot fail at this point
        .unwrap();
    /*
     * we need at least a small common time frame,
     * otherwise analysis is not feasible
     */
    let mut feasible = false;
    if let (Some(first_sp3), Some(last_sp3)) = (sp3.first_epoch(), sp3.last_epoch()) {
        if let (Some(first_nav), Some(last_nav)) = (nav.first_epoch(), nav.last_epoch()) {
            feasible |= (first_nav >= first_sp3) || (first_nav <= last_sp3);
            feasible |= (last_nav >= first_sp3) || (last_nav <= last_sp3);
        }
    }

    if !feasible {
        warn!("|sp3-nav| residual analysis not feasible due to non common time frame");
        return;
    }

    let mut residuals: HashMap<SV, (Vec<(f64, f64, f64)>, Vec<Epoch>)> = HashMap::new();

    for (t, nav_sv, (x_km, y_km, z_km)) in nav.sv_position() {
        if let Some((_, _, (sp3_x, sp3_y, sp3_z))) = sp3
            .sv_position()
            .find(|(e_sp3, sv_sp3, (_, _, _))| *e_sp3 == t && *sv_sp3 == nav_sv)
        {
            /* no need to interpolate => use right away */
            if let Some((residuals, epochs)) = residuals.get_mut(&nav_sv) {
                residuals.push((
                    (x_km - sp3_x) / 1.0E3,
                    (y_km - sp3_y) / 1.0E3,
                    (z_km - sp3_z) / 1.0E3,
                ));
                epochs.push(t);
            } else {
                residuals.insert(
                    nav_sv,
                    (
                        vec![(
                            (x_km - sp3_x) / 1.0E3,
                            (y_km - sp3_y) / 1.0E3,
                            (z_km - sp3_z) / 1.0E3,
                        )],
                        vec![t],
                    ),
                );
            }
        } else {
            /* needs interpolation */
            if let Some((sp3_x, sp3_y, sp3_z)) = sp3.sv_position_interpolate(nav_sv, t, 11) {
                if let Some((residuals, epochs)) = residuals.get_mut(&nav_sv) {
                    residuals.push((
                        (x_km - sp3_x) / 1.0E3,
                        (y_km - sp3_y) / 1.0E3,
                        (z_km - sp3_z) / 1.0E3,
                    ));
                    epochs.push(t);
                } else {
                    residuals.insert(
                        nav_sv,
                        (
                            vec![(
                                (x_km - sp3_x) / 1.0E3,
                                (y_km - sp3_y) / 1.0E3,
                                (z_km - sp3_z) / 1.0E3,
                            )],
                            vec![t],
                        ),
                    );
                }
            }
        }
    }
    /*
     * Plot x residuals
     */
    for (sv_index, (sv, (residuals, epochs))) in residuals.iter().enumerate() {
        if sv_index == 0 {
            plot_ctx.add_timedomain_plot(
                "Broadast Vs SP3 (Post Processed) Residual X errors",
                "x_err [m]",
            );
            trace!("|sp3 - broadcast| residual x error");
        }

        let trace = build_chart_epoch_axis(
            &format!("{:X}", sv),
            Mode::Markers,
            epochs.to_vec(),
            residuals.iter().map(|(x, _, _)| *x).collect::<Vec<f64>>(),
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
    /*
     * plot y residuals
     */
    for (sv_index, (sv, (residuals, epochs))) in residuals.iter().enumerate() {
        if sv_index == 0 {
            plot_ctx.add_timedomain_plot(
                "Broadast Vs SP3 (Post Processed) Residual Y errors",
                "y_err [m]",
            );
            trace!("|sp3 - broadcast| residual y error");
        }

        let trace = build_chart_epoch_axis(
            &format!("{:X}", sv),
            Mode::Markers,
            epochs.to_vec(),
            residuals.iter().map(|(_, y, _)| *y).collect::<Vec<f64>>(),
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
    /*
     * plot z residuals
     */
    for (sv_index, (sv, (residuals, epochs))) in residuals.iter().enumerate() {
        if sv_index == 0 {
            plot_ctx.add_timedomain_plot(
                "Broadast Vs SP3 (Post Processed) Residual Z errors",
                "z_err [m]",
            );
            trace!("|sp3 - broadcast| residual z error");
        }

        let trace = build_chart_epoch_axis(
            &format!("{:X}", sv),
            Mode::Markers,
            epochs.to_vec(),
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
