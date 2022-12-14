use crate::{
    plot::{build_chart_epoch_axis, generate_markers, PlotContext},
    Context,
};
use ndarray::Array;
use plotly::{
    common::{Marker, Mode, Visible},
    Scatter,
};
use rinex::prelude::*;

/*
 * Sv per epoch analysis
 */
pub fn sv_epoch(ctx: &Context, plot_ctx: &mut PlotContext) {
    plot_ctx.add_cartesian2d_plot("Sv per Epoch", "Sv(PRN#)");
    let constellations = ctx.primary_rinex.list_constellations();
    let mut nb_markers = constellations.len();

    if let Some(ref nav) = ctx.nav_rinex {
        let nav_constell = nav.list_constellations();
        nb_markers += nav_constell.len();
    }

    let markers = generate_markers(nb_markers);

    let data = ctx.primary_rinex.space_vehicules_per_epoch();
    for (sv_index, sv) in ctx.primary_rinex.space_vehicules().iter().enumerate() {
        let epochs: Vec<Epoch> = data
            .iter()
            .filter_map(|(epoch, ssv)| {
                if ssv.contains(&sv) {
                    Some(*epoch)
                } else {
                    None
                }
            })
            .collect();
        let constell_index = constellations
            .iter()
            .position(|c| *c == sv.constellation)
            .unwrap();
        let prn = Array::linspace(0.0, 1.0, epochs.len());
        let prn: Vec<f64> = prn
            .iter()
            .map(|_| sv.prn as f64 + constell_index as f64)
            .collect();
        let marker = &markers[constell_index];
        let trace = build_chart_epoch_axis(&sv.to_string(), Mode::Markers, epochs, prn)
            .marker(Marker::new().symbol(marker.clone()))
            .visible({
                // improves plot generation speed, on large files
                if sv_index < 4 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
        plot_ctx.add_trace(trace);
    }

    if let Some(ref nav) = ctx.nav_rinex {
        let data = nav.space_vehicules_per_epoch();
        let nav_constell = nav.list_constellations();
        let nb_obs_constell = nb_markers - nav_constell.len();

        for (sv_index, sv) in nav.space_vehicules().iter().enumerate() {
            let epochs: Vec<Epoch> = data
                .iter()
                .filter_map(|(epoch, ssv)| {
                    if ssv.contains(&sv) {
                        Some(*epoch)
                    } else {
                        None
                    }
                })
                .collect();
            let constell_index = constellations
                .iter()
                .position(|c| *c == sv.constellation)
                .unwrap();
            let prn = Array::linspace(0.0, 1.0, epochs.len());
            let prn: Vec<f64> = prn
                .iter()
                .map(|_| sv.prn as f64 + constell_index as f64 + nb_obs_constell as f64)
                .collect();
            let marker = &markers[constell_index];
            let trace = build_chart_epoch_axis(&format!("{}(NAV)", sv), Mode::Markers, epochs, prn)
                .marker(Marker::new().symbol(marker.clone()))
                .visible({
                    // improves plot generation speed, on large files
                    if sv_index < 4 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                });
            plot_ctx.add_trace(trace);
        }
    }
}
