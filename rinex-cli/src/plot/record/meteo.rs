use crate::plot::{build_chart_epoch_axis, PlotContext}; //generate_markers};
use plotly::common::{Marker, MarkerSymbol, Mode};
use rinex::prelude::*;
use rinex_qc::QcContext;

/*
 * Plots Meteo RINEX
 */
pub fn plot_meteo(ctx: &QcContext, plot_context: &mut PlotContext) {
    /*
     * 1 plot per physics
     */
    for observable in ctx.primary_data().observable() {
        let unit = match observable {
            Observable::Pressure => "hPa",
            Observable::Temperature => "°C",
            Observable::HumidityRate | Observable::RainIncrement => "%",
            Observable::ZenithWetDelay
            | Observable::ZenithDryDelay
            | Observable::ZenithTotalDelay => "s",
            Observable::WindDirection => "°",
            Observable::WindSpeed => "m/s",
            Observable::HailIndicator => "",
            _ => unreachable!(),
        };
        plot_context.add_cartesian2d_plot(
            &format!("{} Observations", observable),
            &format!("{} [{}]", observable, unit),
        );
        let data_x: Vec<_> = ctx
            .primary_data()
            .meteo()
            .flat_map(|(e, observations)| {
                observations.iter().filter_map(
                    |(obs, _value)| {
                        if obs == observable {
                            Some(*e)
                        } else {
                            None
                        }
                    },
                )
            })
            .collect();
        let data_y: Vec<_> = ctx
            .primary_data()
            .meteo()
            .flat_map(|(_e, observations)| {
                observations.iter().filter_map(|(obs, value)| {
                    if obs == observable {
                        Some(*value)
                    } else {
                        None
                    }
                })
            })
            .collect();
        let trace =
            build_chart_epoch_axis(&observable.to_string(), Mode::LinesMarkers, data_x, data_y)
                .marker(Marker::new().symbol(MarkerSymbol::TriangleUp));
        plot_context.add_trace(trace);
    }
}
