use super::{build_chart_epoch_axis, generate_markers, Marker, Mode, PlotContext};
use plotly::common::Visible;
use rinex::prelude::*;
use std::collections::{BTreeMap, HashMap};

/*
 * Plot GNSS recombination
 */
pub fn plot_gnss_recombination(
    plot_context: &mut PlotContext,
    plot_title: &str,
    y_title: &str,
    data: &HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>>,
) {
    // add a plot
    plot_context.add_cartesian2d_plot(plot_title, y_title);

    // generate 1 marker per OP
    let markers = generate_markers(data.len());

    // plot all ops
    for (op_index, ((lhs_observable, ref_observable), vehicles)) in data.iter().enumerate() {
        for (sv, epochs) in vehicles {
            let data_x: Vec<Epoch> = epochs.iter().map(|((e, _flag), _v)| *e).collect();
            let data_y: Vec<f64> = epochs.iter().map(|(_, v)| *v).collect();
            let trace = build_chart_epoch_axis(
                &format!("{}({}-{})", sv, lhs_observable, ref_observable),
                Mode::Markers,
                data_x,
                data_y,
            )
            .marker(Marker::new().symbol(markers[op_index].clone()))
            .visible({
                if op_index < 1 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
            plot_context.add_trace(trace);
        }
    }
}

/*
 * Plot DCB analysis
 */
pub fn plot_gnss_dcb(
    plot_context: &mut PlotContext,
    plot_title: &str,
    y_title: &str,
    data: &HashMap<String, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>>,
) {
    // add a plot
    plot_context.add_cartesian2d_plot(plot_title, y_title);
    // generate 1 marker per OP
    let markers = generate_markers(data.len());
    // plot all ops
    for (op_index, (op, vehicles)) in data.iter().enumerate() {
        for (_sv, epochs) in vehicles {
            let data_x: Vec<Epoch> = epochs.iter().map(|((e, _flag), _v)| *e).collect();
            let data_y: Vec<f64> = epochs.iter().map(|(_, v)| *v).collect();
            let trace = build_chart_epoch_axis(&op, Mode::Markers, data_x, data_y)
                .marker(Marker::new().symbol(markers[op_index].clone()))
                .visible({
                    if op_index < 1 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                });
            plot_context.add_trace(trace);
        }
    }
}

/*
 * Plots Ionospheric delay detector
 */
pub fn plot_iono_detector(
    plot_context: &mut PlotContext,
    data: &HashMap<Observable, HashMap<SV, BTreeMap<Epoch, f64>>>,
) {
    // add a plot
    plot_context.add_cartesian2d_plot(
        "Ionospheric Delay Detector",
        "Variations of Meters of delay",
    );
    // generate 1 marker per OP
    let markers = generate_markers(data.len());
    // plot all ops
    for (op_index, (op, vehicles)) in data.iter().enumerate() {
        for (sv, epochs) in vehicles {
            let data_x: Vec<Epoch> = epochs.iter().map(|(e, _v)| *e).collect();
            let data_y: Vec<f64> = epochs.iter().map(|(_, v)| *v).collect();
            let trace =
                build_chart_epoch_axis(&format!("{}({})", sv, op), Mode::Markers, data_x, data_y)
                    .marker(Marker::new().symbol(markers[op_index].clone()))
                    .visible({
                        if op_index < 1 {
                            Visible::True
                        } else {
                            Visible::LegendOnly
                        }
                    });
            plot_context.add_trace(trace);
        }
    }
}
