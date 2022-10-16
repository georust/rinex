//! Meteo observations plotting
use rinex::meteo::*;
use super::Context;
use plotters::prelude::*;
use std::collections::HashMap;

pub fn plot(ctx: &mut Context, record: &Record) {
    let mut t0 : i64 = 0;
    let mut datasets: HashMap<String, Vec<(f64, f64)>> = HashMap::new();
    for (index, (epoch, observations)) in record.iter().enumerate() {
        if index == 0 {
            t0 = epoch.date.timestamp();
        }
        let t = epoch.date.timestamp();
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
        let mut chart = ctx.charts
            .get(&observable)
            .expect(&format!("faulty context, expecting a chart dedicated to \"{}\" observable", observable))
            .clone()
            .restore(ctx.plots.get(&observable.to_string()).unwrap());
        chart
            .draw_series(LineSeries::new(
                data.iter()
                    .map(|(x, y)| (*x, *y)),
                    &BLACK
                ))
            .expect(&format!("failed to draw {} chart", observable))
            .label(observable)
            .legend(|(x, y)| {
                //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
                PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
            });
        chart
            .draw_series(data.iter()
                .map(|point| Cross::new(*point, 4, BLACK.filled())))
                .unwrap();
        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(WHITE.filled())
            .draw()
            .expect("failed to draw labels on chart");
    }
} 
