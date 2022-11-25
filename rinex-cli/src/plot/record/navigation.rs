//! Navigation record plotting
use rinex::{
    prelude::*,
    navigation::*,
};
use plotters::{
    prelude::*,
    coord::Shift,
    chart::ChartState,
};
use super::{
    Context, Plot2d,
    build_plot, build_chart,
};
use std::collections::HashMap;

pub fn build_context<'a> (dim: (u32, u32), record: &Record) -> Context<'a> {
    let mut e0: f64 = 0.0;
    let mut t_axis: Vec<f64> = Vec::with_capacity(16384);
    let mut plots: HashMap<String,
        DrawingArea<BitMapBackend, Shift>>
            = HashMap::with_capacity(4);
    let _y_ranges: HashMap<String, (f64,f64)> = HashMap::new();
    let mut charts: HashMap<String, ChartState<Plot2d>> = HashMap::new();
    for (index, (e, classes)) in record.iter().enumerate() {
        if index == 0 {
            // store first epoch timestamp
            // to scale x_axis proplery (avoids fuzzy rendering)
            e0 = e.to_utc_seconds();
        }
        let t = e.to_utc_seconds() - e0;
        t_axis.push(t as f64);
        for (class, _) in classes {
            if *class == FrameClass::Ephemeris {
                let file = "clock-bias.png";
                if plots.get(file).is_none() {
                    let plot = build_plot(file, dim);
                    plots.insert(file.to_string(), plot);
                }
                let file = "clock-drift.png";
                if plots.get(file).is_none() {
                    let plot = build_plot(file, dim);
                    plots.insert(file.to_string(), plot);
                }
                let file = "clock-driftr.png";
                if plots.get(file).is_none() {
                    let plot = build_plot(file, dim);
                    plots.insert(file.to_string(), plot);
                }
            } else {
                println!("Non ephemeris frame cannot be plotted yet");
            }
        }
    }
    // Add one chart onto all plots
    for (id, plot) in plots.iter() {
        // scale this chart nicely
        let chart = build_chart(id, t_axis.clone(), (-10.0E5, 10E5), plot);
        charts.insert(id.to_string(), chart);
    }
    Context {
        t_axis,
        plots,
        charts,
    }
}

pub fn plot(ctx: &mut Context, record: &Record) {
    let mut e0: f64 = 0.0;
    let mut bias: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
    let mut drift: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
    let mut driftr: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
    for (index, (epoch, classes)) in record.iter().enumerate() {
        if index == 0 {
            e0 = epoch.to_utc_seconds();
        }
        let t = epoch.to_utc_seconds() - e0;
        for (class, frames) in classes {
            if *class == FrameClass::Ephemeris {
                for frame in frames {
                    if let Some((_, sv, eph)) = frame.as_eph() {
                        if let Some(bias) = bias.get_mut(&sv) {
                            bias.push((t as f64, eph.clock_bias));
                        } else {
                            bias.insert(*sv, vec![(t as f64, eph.clock_bias)]);
                        }
                        if let Some(drift) = drift.get_mut(&sv) {
                            drift.push((t as f64, eph.clock_drift));
                        } else {
                            drift.insert(*sv, vec![(t as f64, eph.clock_drift)]);
                        }
                        if let Some(driftr) = driftr.get_mut(&sv) {
                            driftr.push((t as f64, eph.clock_drift_rate));
                        } else {
                            driftr.insert(*sv, vec![(t as f64, eph.clock_drift_rate)]);
                        }
                    }
                }
            }
        }
    }
    let plot = ctx.plots.get("clock-bias.png")
        .unwrap();
    let mut chart = ctx.charts.get("clock-bias.png")
        .unwrap()
        .clone()
        .restore(&plot);
    for (_vehicule, bias) in bias {
        chart
            .draw_series(LineSeries::new(
                bias.iter().map(|point| *point),
                &BLACK,
            ))
            .expect("failed to draw clock biases")
            .label("Clock bias")
            .legend(|(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
            });
    }
} 
