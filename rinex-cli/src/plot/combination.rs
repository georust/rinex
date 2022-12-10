use super::{
    Context,
    generate_markers,
    build_default_plot,
    Marker,
    Scatter,
    Mode,
};
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
    let mut plot = build_default_plot(plot_title, y_title); 
    let markers = generate_markers(data.len()); // one marker per op
    // plot all ops
    for (op_index, (op, vehicules)) in data.iter().enumerate() {
        for (sv, epochs) in vehicules {
            let data_x: Vec<String> = epochs.iter()
                .map(|((e, _flag), _v)| e.to_string())
                .collect();
            let data_y: Vec<f64> = epochs.iter()
                .map(|(_, v)| *v)
                .collect();
            let trace = Scatter::new(data_x, data_y)
                .mode(Mode::Markers)
                .marker(
                    Marker::new()
                        .symbol(markers[op_index].clone())
                )
                .name(&format!("{}({})", sv, op));
            plot.add_trace(trace);
        }
    }
    plot.show();
}
