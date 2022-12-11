use super::{generate_markers, Context, Marker, Mode, Scatter};
use plotly::common::Visible;
use rinex::prelude::*;
use std::collections::{BTreeMap, HashMap};

/*
 * Plots (any kind of) recombined GNSS dataset
 */
pub fn plot_gnss_recombination(
    ctx: &mut Context,
    plot_title: &str,
    y_title: &str,
    data: &HashMap<String, HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>,
) {
    // add a plot
    ctx.add_cartesian2d_plot(plot_title, y_title);
    // generate 1 marker per OP
    let markers = generate_markers(data.len());
    // plot all ops
    for (op_index, (op, vehicules)) in data.iter().enumerate() {
        for (sv, epochs) in vehicules {
            let data_x: Vec<String> = epochs
                .iter()
                .map(|((e, _flag), _v)| e.to_string())
                .collect();
            let data_y: Vec<f64> = epochs.iter().map(|(_, v)| *v).collect();
            let trace = Scatter::new(data_x, data_y)
                .mode(Mode::Markers)
                .marker(Marker::new().symbol(markers[op_index].clone()))
                .web_gl_mode(true)
                .visible({
                    if op_index < 1 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                })
                .name(&format!("{}({})", sv, op));
            ctx.add_trace(trace);
        }
    }
}
