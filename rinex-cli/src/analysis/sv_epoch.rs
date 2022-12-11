use crate::plot::{generate_markers, Context};
use ndarray::Array;
use plotly::{
    common::{Marker, Mode, Visible},
    Scatter,
};
use rinex::prelude::*;

/*
 * Sv per epoch analysis
 */
pub fn sv_epoch(ctx: &mut Context, rnx: &Rinex, nav: &mut Option<Rinex>) {
    // create a new plot
    ctx.add_cartesian2d_plot("Sv per Epoch", "Sv");
    let constellations = rnx.list_constellations();
    let mut nb_markers = constellations.len();

    if let Some(nav) = nav {
        let nav_constell = nav.list_constellations();
        nb_markers += nav_constell.len();
    }

    let markers = generate_markers(nb_markers);

    let data = rnx.space_vehicules_per_epoch();
    for (sv_index, sv) in rnx.space_vehicules().iter().enumerate() {
        let epochs: Vec<String> = data
            .iter()
            .filter_map(|(epoch, ssv)| {
                if ssv.contains(&sv) {
                    Some(epoch.to_string())
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
        let prn: Vec<u8> = prn.iter().map(|_| sv.prn + constell_index as u8).collect();
        let marker = &markers[constell_index];
        let trace = Scatter::new(epochs, prn)
            .mode(Mode::Markers)
            .marker(Marker::new().symbol(marker.clone()))
            .visible({
                // improves plot generation speed, on large files
                if sv_index < 4 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            })
            .name(&sv.to_string());
        ctx.add_trace(trace);
    }

    if let Some(nav) = nav {
        let data = nav.space_vehicules_per_epoch();
        let nav_constell = nav.list_constellations();
        let nb_obs_constell = nb_markers - nav_constell.len();

        for (sv_index, sv) in rnx.space_vehicules().iter().enumerate() {
            let epochs: Vec<String> = data
                .iter()
                .filter_map(|(epoch, ssv)| {
                    if ssv.contains(&sv) {
                        Some(epoch.to_string())
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
            let prn: Vec<u8> = prn
                .iter()
                .map(|_| sv.prn + constell_index as u8 + nb_obs_constell as u8)
                .collect();
            let marker = &markers[constell_index];
            let trace = Scatter::new(epochs, prn)
                .mode(Mode::Markers)
                .marker(Marker::new().symbol(marker.clone()))
                .visible({
                    // improves plot generation speed, on large files
                    if sv_index < 4 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                })
                .name(&format!("{}(NAV)", sv));
            ctx.add_trace(trace);
        }
    }
}
