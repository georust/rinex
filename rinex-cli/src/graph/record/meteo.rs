use crate::cli::Context;
use crate::graph::{build_chart_epoch_axis, csv_export_timedomain, PlotContext}; //generate_markers};
use plotly::common::{Marker, MarkerSymbol, Mode};
use plotly::ScatterPolar;
use rinex::prelude::Observable;
use statrs::statistics::Statistics;

/*
 * Plots Meteo observations
 */
pub fn plot_meteo_observations(ctx: &Context, plot_context: &mut PlotContext, csv_export: bool) {
    let rnx = ctx.data.meteo_data().unwrap(); // infaillible
                                              /*
                                               * 1 plot per physics
                                               */
    for observable in rnx.observable() {
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
        if *observable == Observable::WindDirection {
            // we plot this one differently: on a compass similar to skyplot
            continue;
        }
        plot_context.add_timedomain_plot(
            &format!("{} Observations", observable),
            &format!("{} [{}]", observable, unit),
        );
        let data_x: Vec<_> = rnx
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
        let data_y: Vec<_> = rnx
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
        let trace = build_chart_epoch_axis(
            &observable.to_string(),
            Mode::LinesMarkers,
            data_x.clone(),
            data_y.clone(),
        )
        .marker(Marker::new().symbol(MarkerSymbol::TriangleUp));
        plot_context.add_trace(trace);
        if csv_export {
            let fullpath = ctx
                .workspace
                .join("CSV")
                .join(&format!("{}.csv", observable));

            let title = format!("{} observations", observable);
            csv_export_timedomain(
                &fullpath,
                &title,
                &format!("Epoch, {} [{}]", observable, unit),
                &data_x,
                &data_y,
            )
            .expect("failed to render data as CSV");
        }
    }
    /*
     * Plot Wind Direction
     */
    let wind_speed = rnx.wind_speed().map(|(_, speed)| speed).collect::<Vec<_>>();
    let wind_speed_max = wind_speed.max();

    let theta = rnx
        .wind_direction()
        .map(|(_, angle)| angle)
        .collect::<Vec<_>>();

    let has_wind_direction = !theta.is_empty();

    let rho = rnx
        .wind_direction()
        .map(|(t, _)| {
            if let Some(speed) = rnx
                .wind_speed()
                .find(|(ts, _)| *ts == t)
                .map(|(_, speed)| speed)
            {
                speed / wind_speed_max
            } else {
                1.0_f64
            }
        })
        .collect::<Vec<_>>();

    let trace = ScatterPolar::new(theta, rho)
        .marker(Marker::new().symbol(MarkerSymbol::TriangleUp))
        .connect_gaps(false)
        .name("Wind direction [°]");
    if has_wind_direction {
        plot_context.add_polar2d_plot("Wind direction (r= normalized speed)");
        plot_context.add_trace(trace);
    }
}
