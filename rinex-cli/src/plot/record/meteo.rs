use rinex::{
    prelude::*,
    meteo::*,
};
use std::collections::HashMap;
use crate::plot::{build_default_plot, generate_markers};
use plotly::{
    Scatter,
    common::{Marker, MarkerSymbol, Mode},
};

/*
 * Plots Meteo RINEX
 */
pub fn plot_meteo(record: &Record) {
    /*
     * 1 plot per physics
     */
    let mut datasets: HashMap<String, Vec<(String, f64)>> = HashMap::new();
    for (index, (epoch, observations)) in record.iter().enumerate() {
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
            "PR" => "Bar",
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
        let mut plot = build_default_plot(
            &format!("{} Observations", observable),
            &format!("{} [{}]", observable, unit));
        let data_x: Vec<String> = data
            .iter()
            .map(|(k, _v)| k.clone())
            .collect();
        let data_y: Vec<f64> = data
            .iter()
            .map(|(k, v)| *v)
            .collect();
        let trace = Scatter::new(data_x, data_y)
            .mode(Mode::LinesMarkers)
            .marker(
                Marker::new()
                    .symbol(MarkerSymbol::TriangleUp)
            )
            .name(observable);
        plot.add_trace(trace);
        plot.show();
    }
}
