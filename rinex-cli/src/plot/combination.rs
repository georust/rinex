use super::{build_chart_epoch_axis, generate_markers, Marker, Mode, PlotContext};
use plotly::common::Visible;
use rinex::prelude::*;
use std::collections::{BTreeMap, HashMap};

/*
 * Plot GNSS recombination
 */
pub fn plot_gnss_recombination(
    ctx: &mut PlotContext,
    plot_title: &str,
    y_title: &str,
    data: &HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>,
) {
    // add a plot
    ctx.add_cartesian2d_plot(plot_title, y_title);
    // generate 1 marker per OP
    let markers = generate_markers(data.len());
    // plot all ops
    for (op_index, ((lhs_observable, ref_observable), vehicules)) in data.iter().enumerate() {
        for (sv, epochs) in vehicules {
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
            ctx.add_trace(trace);
        }
    }
}

/*
 * Plot DCB analysis
 */
pub fn plot_gnss_dcb(
    ctx: &mut PlotContext,
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
        for (_sv, epochs) in vehicules {
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
            ctx.add_trace(trace);
        }
    }
}

/*
 * Plots Ionospheric delay detector
 */
pub fn plot_iono_detector(
    ctx: &mut PlotContext,
    data: &HashMap<Observable, HashMap<Sv, BTreeMap<Epoch, f64>>>,
) {
    // add a plot
    ctx.add_cartesian2d_plot(
        "Ionospheric Delay Detector",
        "Variations of Meters of delay",
    );
    // generate 1 marker per OP
    let markers = generate_markers(data.len());
    // plot all ops
    for (op_index, (op, vehicules)) in data.iter().enumerate() {
        for (sv, epochs) in vehicules {
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
            ctx.add_trace(trace);
        }
    }
}
