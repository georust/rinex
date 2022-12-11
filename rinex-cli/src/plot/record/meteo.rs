use crate::plot::Context; //generate_markers};
use plotly::{
    common::{Marker, MarkerSymbol, Mode},
    Scatter,
};
use rinex::meteo::*;
use std::collections::HashMap;

/*
 * Plots Meteo RINEX
 */
pub fn plot_meteo(ctx: &mut Context, record: &Record) {
    /*
     * 1 plot per physics
     */
    let mut datasets: HashMap<String, Vec<(String, f64)>> = HashMap::new();
    for (epoch, observations) in record {
        for (observable, observation) in observations {
            if let Some(data) = datasets.get_mut(&observable.to_string()) {
                data.push((epoch.to_string(), *observation));
            } else {
                datasets.insert(
                    observable.to_string(),
                    vec![(epoch.to_string(), *observation)],
                );
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
        let data_x: Vec<String> = data.iter().map(|(k, _)| k.clone()).collect();
        let data_y: Vec<f64> = data.iter().map(|(_, v)| *v).collect();
        let trace = Scatter::new(data_x, data_y)
            .mode(Mode::LinesMarkers)
            .marker(Marker::new().symbol(MarkerSymbol::TriangleUp))
            .name(observable);
        ctx.add_trace(trace);
    }
}
