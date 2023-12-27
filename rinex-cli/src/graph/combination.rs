use super::{build_chart_epoch_axis, generate_markers, Marker, Mode, PlotContext};
use plotly::common::Visible;
use rinex::prelude::*;
use std::collections::{BTreeMap, HashMap};

pub fn plot_gnss_combination(
    data: &HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>>,
    plot_context: &mut PlotContext,
    plot_title: &str,
    y_title: &str,
) {
    // add a plot
    plot_context.add_timedomain_plot(plot_title, y_title);

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
                if op_index < 2 {
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
    data: &HashMap<String, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>>,
    plot_context: &mut PlotContext,
    plot_title: &str,
    y_title: &str,
) {
    // add a plot
    plot_context.add_timedomain_plot(plot_title, y_title);
    // generate 1 marker per OP
    let markers = generate_markers(data.len());
    // plot all ops
    for (op_index, (op, vehicles)) in data.iter().enumerate() {
        for (_sv, epochs) in vehicles {
            let data_x: Vec<Epoch> = epochs.iter().map(|((e, _flag), _v)| *e).collect();
            let data_y: Vec<f64> = epochs.iter().map(|(_, v)| *v).collect();
            let trace = build_chart_epoch_axis(&op.to_string()[1..], Mode::Markers, data_x, data_y)
                .marker(Marker::new().symbol(markers[op_index].clone()))
                .visible({
                    if op_index < 2 {
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
 * Plot MP analysis
 */
pub fn plot_gnss_code_mp(
    data: &HashMap<Observable, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>>,
    plot_context: &mut PlotContext,
    plot_title: &str,
    y_title: &str,
) {
    // add a plot
    plot_context.add_timedomain_plot(plot_title, y_title);
    // generate 1 marker per OP
    let markers = generate_markers(data.len());
    // plot all ops
    for (op_index, (op, vehicles)) in data.iter().enumerate() {
        for (_sv, epochs) in vehicles {
            let data_x: Vec<Epoch> = epochs.iter().map(|((e, _flag), _v)| *e).collect();
            let data_y: Vec<f64> = epochs.iter().map(|(_, v)| *v).collect();
            let trace = build_chart_epoch_axis(&op.to_string()[1..], Mode::Markers, data_x, data_y)
                .marker(Marker::new().symbol(markers[op_index].clone()))
                .visible({
                    if op_index < 2 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                });
            plot_context.add_trace(trace);
        }
    }
}
