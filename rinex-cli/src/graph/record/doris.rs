use crate::cli::Context;
use itertools::Itertools;
use plotly::common::{Marker, MarkerSymbol, Mode, Visible};
use rinex::{carrier::Carrier, prelude::*};

use crate::graph::{build_chart_epoch_axis, PlotContext};

fn plot_title(observable: Observable) -> (String, String) {
    if observable.is_pseudorange_observable() {
        ("Pseudo Range".to_string(), "[m]".to_string())
    } else if observable.is_phase_observable() {
        ("Phase".to_string(), "[m]".to_string())
    } else if observable.is_power_observable() {
        ("Signal Power".to_string(), "Power [dBm]".to_string())
    } else if observable == Observable::Temperature {
        (
            "Temperature (at base station)".to_string(),
            "T [°C]".to_string(),
        )
    } else if observable == Observable::Pressure {
        (
            "Pressure (at base station)".to_string(),
            "P [hPa]".to_string(),
        )
    } else if observable == Observable::HumidityRate {
        (
            "Humidity (at base station)".to_string(),
            "Saturation Rate [%]".to_string(),
        )
    } else if observable == Observable::FrequencyRatio {
        (
            "RX Clock Offset (f(t)-f0)/f0".to_string(),
            "n.a".to_string(),
        )
    } else {
        unreachable!("unexpected DORIS observable {}", observable);
    }
}

/*
 * Plots given DORIS RINEX content
 */
pub fn plot_doris_observations(ctx: &Context, plot_ctx: &mut PlotContext, _csv_export: bool) {
    let doris = ctx.data.doris().unwrap(); // infaillible

    // Per observable
    for observable in doris.observable().sorted() {
        // Trick to present S1/U2 on the same plot
        let marker_symbol = if observable.is_phase_observable()
            || observable.is_pseudorange_observable()
            || observable.is_power_observable()
        {
            let obs_str = observable.to_string();
            if obs_str.contains("1") {
                let (plot_title, y_title) = plot_title(observable.clone());
                plot_ctx.add_timedomain_plot(&plot_title, &y_title);
                MarkerSymbol::Circle
            } else {
                MarkerSymbol::Diamond
            }
        } else {
            let (plot_title, y_title) = plot_title(observable.clone());
            plot_ctx.add_timedomain_plot(&plot_title, &y_title);
            MarkerSymbol::Circle
        };

        // Per station
        for (station_index, station) in doris.stations().sorted().enumerate() {
            if *observable == Observable::Temperature {
                let x = doris
                    .doris_temperature()
                    .filter_map(|(t_i, station_i, _data)| {
                        if station_i == station {
                            Some(t_i)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let y = doris
                    .doris_temperature()
                    .filter_map(|(_t_i, station_i, value)| {
                        if station_i == station {
                            Some(value)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let trace = build_chart_epoch_axis(&station.label, Mode::Markers, x, y)
                    .marker(Marker::new().symbol(marker_symbol.clone()))
                    .visible({
                        if station_index < 3 {
                            Visible::True
                        } else {
                            Visible::LegendOnly
                        }
                    });
                plot_ctx.add_trace(trace);
            } else if *observable == Observable::Pressure {
                let x = doris
                    .doris_pressure()
                    .filter_map(|(t_i, station_i, _data)| {
                        if station_i == station {
                            Some(t_i)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let y = doris
                    .doris_pressure()
                    .filter_map(|(_t_i, station_i, value)| {
                        if station_i == station {
                            Some(value)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let trace = build_chart_epoch_axis(&station.label, Mode::Markers, x, y)
                    .marker(Marker::new().symbol(marker_symbol.clone()))
                    .visible({
                        if station_index < 3 {
                            Visible::True
                        } else {
                            Visible::LegendOnly
                        }
                    });
                plot_ctx.add_trace(trace);
            } else if *observable == Observable::HumidityRate {
                let x = doris
                    .doris_humidity()
                    .filter_map(|(t_i, station_i, _data)| {
                        if station_i == station {
                            Some(t_i)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let y = doris
                    .doris_humidity()
                    .filter_map(|(_t_i, station_i, value)| {
                        if station_i == station {
                            Some(value)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let trace = build_chart_epoch_axis(&station.label, Mode::Markers, x, y)
                    .marker(Marker::new().symbol(marker_symbol.clone()))
                    .visible({
                        if station_index < 3 {
                            Visible::True
                        } else {
                            Visible::LegendOnly
                        }
                    });
                plot_ctx.add_trace(trace);
            // TODO
            // } else if *observable == Observable::FrequencyRatio {
            } else if observable.is_power_observable() {
                let x = doris
                    .doris_rx_power()
                    .filter_map(|(t_i, station_i, observable_i, _data)| {
                        if station_i == station && observable_i == observable {
                            Some(t_i)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let y = doris
                    .doris_rx_power()
                    .filter_map(|(_t_i, station_i, observable_i, data)| {
                        if station_i == station && observable_i == observable {
                            Some(data)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                let freq_label = Carrier::from_doris_observable(&observable)
                    .unwrap_or_else(|_| {
                        panic!("failed to determine plot freq_label for {}", observable)
                    })
                    .to_string();

                let trace = build_chart_epoch_axis(
                    &format!("{}({})", station.label, freq_label),
                    Mode::Markers,
                    x,
                    y,
                )
                .marker(Marker::new().symbol(marker_symbol.clone()))
                .visible({
                    if station_index < 3 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                });
                plot_ctx.add_trace(trace);
            } else if observable.is_pseudorange_observable() {
                let x = doris
                    .doris_pseudo_range()
                    .filter_map(|(t_i, station_i, observable_i, _data)| {
                        if station_i == station && observable_i == observable {
                            Some(t_i)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let y = doris
                    .doris_pseudo_range()
                    .filter_map(|(_t_i, station_i, observable_i, data)| {
                        if station_i == station && observable_i == observable {
                            Some(data)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                let freq_label = Carrier::from_doris_observable(&observable)
                    .unwrap_or_else(|_| {
                        panic!("failed to determine plot freq_label for {}", observable)
                    })
                    .to_string();

                let trace = build_chart_epoch_axis(
                    &format!("{}({})", station.label, freq_label),
                    Mode::Markers,
                    x,
                    y,
                )
                .marker(Marker::new().symbol(marker_symbol.clone()))
                .visible({
                    if station_index < 3 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                });
                plot_ctx.add_trace(trace);
            } else if observable.is_phase_observable() {
                let x = doris
                    .doris_phase()
                    .filter_map(|(t_i, station_i, observable_i, _data)| {
                        if station_i == station && observable_i == observable {
                            Some(t_i)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let y = doris
                    .doris_phase()
                    .filter_map(|(_t_i, station_i, observable_i, data)| {
                        if station_i == station && observable_i == observable {
                            Some(data)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                let freq_label = Carrier::from_doris_observable(&observable)
                    .unwrap_or_else(|_| {
                        panic!("failed to determine plot freq_label for {}", observable)
                    })
                    .to_string();

                let trace = build_chart_epoch_axis(
                    &format!("{}({})", station.label, freq_label),
                    Mode::Markers,
                    x,
                    y,
                )
                .marker(Marker::new().symbol(marker_symbol.clone()))
                .visible({
                    if station_index < 3 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                });
                plot_ctx.add_trace(trace);
            }
        }
    }
}
