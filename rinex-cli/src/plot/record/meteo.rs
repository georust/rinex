use crate::plot::{build_chart_epoch_axis, Context}; //generate_markers};
use plotly::common::{Marker, MarkerSymbol, Mode};
use rinex::{meteo::*, prelude::*};
use std::collections::HashMap;

/*
 * Plots Meteo RINEX
 */
pub fn plot_meteo(ctx: &mut Context, record: &Record) {
    /*
     * 1 plot per physics
     */
    let mut datasets: HashMap<String, Vec<(Epoch, f64)>> = HashMap::new();
    for (epoch, observations) in record {
        for (observable, observation) in observations {
            if let Some(data) = datasets.get_mut(&observable.to_string()) {
                data.push((*epoch, *observation));
            } else {
                datasets.insert(observable.to_string(), vec![(*epoch, *observation)]);
            }
        }
    }

    for (observable, data) in datasets {
        let unit = match observable.as_str() {
            "PR" => "hPa",
            "TD" => "°C",
            "HR" => "%",
            "ZW" => "s",
            "ZD" => "s",
            "ZT" => "s",
            "WD" => "°",
            "WS" => "m/s",
            "RI" => "%",
            "HI" => "",
            _ => unreachable!(),
        };
        ctx.add_cartesian2d_plot(
            &format!("{} Observations", observable),
            &format!("{} [{}]", observable, unit),
        );
        let data_x: Vec<Epoch> = data.iter().map(|(k, _)| *k).collect();
        let data_y: Vec<f64> = data.iter().map(|(_, v)| *v).collect();
        let trace = build_chart_epoch_axis(&observable, Mode::LinesMarkers, data_x, data_y)
            .marker(Marker::new().symbol(MarkerSymbol::TriangleUp));
        ctx.add_trace(trace);
    }
}
