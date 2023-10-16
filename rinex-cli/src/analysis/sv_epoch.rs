use crate::plot::{build_chart_epoch_axis, generate_markers, PlotContext};
use ndarray::Array;
use plotly::common::{Marker, Mode, Title, Visible};
use plotly::layout::Axis;
use rinex::prelude::RnxContext;
use rinex::prelude::*;

/*
 * Sv periepoch analysis
pub fn sv_epoch(ctx: &RnxContext, plot_ctx: &mut PlotContext) {
    plot_ctx.add_cartesian2d_plot("Sv per Epoch", "Sv(PRN#)");
     * plot customization
     * We're plotting PRN#, set dy to +/- 1
     * for nicer rendition
    let plot_item = plot_ctx.plot_item_mut().unwrap();
    let layout = plot_item.layout().clone().y_axis(
        Axis::new()
            .title(Title::new("PRN#"))
            .zero_line(false)
            .dtick(1.0),
    );
    plot_item.set_layout(layout);

    // Design markers / symbols
    //   one per constellation system
    let constellations: Vec<_> = ctx.primary_data().constellation().collect();
    let mut nb_markers = constellations.len();

    if let Some(nav) = ctx.navigation_data() {
        nb_markers += nav.constellation().count();
    }

    let markers = generate_markers(nb_markers);

    let data: Vec<_> = ctx.primary_data().sv_epoch().collect();

    for (sv_index, sv) in ctx.primary_data().sv().enumerate() {
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
        let prn: Vec<f64> = prn.iter().map(|_| sv.prn as f64).collect();
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

    if let Some(nav) = &ctx.navigation_data() {
        let data: Vec<_> = nav.sv_epoch().collect();
        let nav_constell: Vec<_> = nav.constellation().collect();

        for (sv_index, sv) in nav.sv().enumerate() {
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
            let constell_index = nav_constell
                .iter()
                .position(|c| *c == sv.constellation)
                .unwrap();
            let prn = Array::linspace(0.0, 1.0, epochs.len());
            let prn: Vec<f64> = prn.iter().map(|_| sv.prn as f64).collect();
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
 */
