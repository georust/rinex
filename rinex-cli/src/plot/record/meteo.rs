//! Meteo observations plotting
use rinex::meteo::*;
use super::{
    Context, Plot2d,
    build_plot, build_chart,
};
use plotters::{
    prelude::*,
    coord::Shift,
    chart::ChartState,
};
use std::collections::HashMap;

/*
 * Builds a plot context for Observation RINEX specificly
 */
pub fn build_context<'a> (dim: (u32, u32), record: &Record) -> Context<'a> {
    let mut e0: f64 = 0.0;
    let mut t_axis: Vec<f64> = Vec::with_capacity(16384);
    let mut plots: HashMap<String,
        DrawingArea<BitMapBackend, Shift>>
            = HashMap::with_capacity(4);
    let mut y_ranges: HashMap<String, (f64,f64)> = HashMap::new();
    let mut charts: HashMap<String, ChartState<Plot2d>> = HashMap::new();
    //  => 1 plot per physics (ie., Observable)
    for (index, (e, observables)) in record.iter().enumerate() {
        if index == 0 {
            // store first epoch timestamp
            // to scale x_axis proplery (avoids fuzzy rendering)
            e0 = e.to_utc_seconds();
        }
        let t = e.to_utc_seconds() - e0;
        t_axis.push(t as f64);
        for (observable, data) in observables {
            if plots.get(&observable.to_string()).is_none() {
                let title = match observable {
                    Observable::Pressure => "pressure.png",
                    Observable::Temperature => "temperature.png",
                    Observable::HumidityRate => "moisture.png",
                    Observable::ZenithWetDelay => "zenith-wet.png",
                    Observable::ZenithDryDelay => "zenith-dry.png",
                    Observable::ZenithTotalDelay => "zenith-total.png",
                    Observable::WindAzimuth => "wind-azim.png",
                    Observable::WindSpeed => "wind-speed.png",
                    Observable::RainIncrement => "rain-increment.png",
                    Observable::HailIndicator=> "hail.png",
                };
                let plot = build_plot(title, dim);
                plots.insert(observable.to_string(), plot);
                y_ranges.insert(observable.to_string(), (*data, *data));
            } else {
                if let Some((min,max)) = y_ranges.get_mut(&observable.to_string()) {
                    if data < min {
                        *min = *data;
                    }
                    if data > max {
                        *max = *data;
                    }
                } else {
                    y_ranges.insert(observable.to_string(), (*data, *data));
                }
            }
        }
    }
    // Add 1 chart onto each plot
    for (id, plot) in plots.iter() {
        // scale this chart nicely
        let range = y_ranges.get(id)
            .unwrap();
        let chart = build_chart(id, t_axis.clone(), *range, plot);
        charts.insert(id.to_string(), chart);
    }
    Context {
        plots,
        charts,
        t_axis,
    }
}


pub fn plot(ctx: &mut Context, record: &Record) {
    let mut t0 : f64 = 0.0;
    let mut datasets: HashMap<String, Vec<(f64, f64)>> = HashMap::new();
    for (index, (epoch, observations)) in record.iter().enumerate() {
        if index == 0 {
            t0 = epoch.to_utc_seconds();
        }
        let t = epoch.to_utc_seconds();
        for (observable, observation) in observations {
            if let Some(data) = datasets.get_mut(&observable.to_string()) {
                data.push(((t-t0) as f64, *observation));
            } else {
                datasets.insert(observable.to_string(),
                    vec![((t-t0) as f64, *observation)]);
            }
        }
    }

    for (observable, data) in datasets {
        let plot = ctx.plots.get(&observable.to_string())
            .expect(&format!("faulty context: missing {} plot", observable));
        let mut chart = ctx.charts
            .get(&observable)
            .expect(&format!("faulty context: missing {} chart", observable))
            .clone()
            .restore(plot);
        chart
            .draw_series(LineSeries::new(
                data.iter()
                    .map(|point| *point),
                &BLACK,
            ))
            .expect(&format!("failed to plot {} data", observable.clone())) 
            .label(observable.clone())
            .legend(move |point| {
                TriangleMarker::new(point, 4, Into::<ShapeStyle>::into(&BLACK).filled())
                .into_dyn()
            });
        chart
            .draw_series(data.iter()
                .map(|point| TriangleMarker::new(*point, 4, BLACK.filled())))
                .expect(&format!("failed to plot {} data", observable)); 
        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(WHITE.filled())
            .draw()
            .expect(&format!("failed to draw labels on {} chart", observable));
    }
} 
