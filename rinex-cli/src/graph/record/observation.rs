use crate::cli::Context;
use plotly::common::{Marker, MarkerSymbol, Mode, Visible};
use std::collections::HashMap;

use rinex::{navigation::Ephemeris, observation::*, prelude::*};

use crate::graph::{build_chart_epoch_axis, csv_export_timedomain, generate_markers, PlotContext};

fn observable_to_physics(observable: &Observable) -> String {
    if observable.is_phase_observable() {
        "Phase".to_string()
    } else if observable.is_doppler_observable() {
        "Doppler".to_string()
    } else if observable.is_ssi_observable() {
        "Signal Strength".to_string()
    } else {
        "Pseudo Range".to_string()
    }
}

/*
 * Plots given Observation RINEX content
 */
pub fn plot_observations(ctx: &Context, plot_context: &mut PlotContext, csv_export: bool) {
    let obs_data = ctx.data.observation().unwrap(); // infaillible

    let header = &obs_data.header;

    let record = obs_data.record.as_obs().unwrap(); // infaillible

    let mut clk_offset: Vec<(Epoch, f64)> = Vec::new();
    // dataset
    //  per physics,
    //   per observable (symbolized)
    //      per vehicle (color map)
    //      bool: loss of lock - CS emphasis
    //      x: sampling timestamp,
    //      y: observation (raw),
    let mut dataset: HashMap<String, HashMap<String, HashMap<SV, Vec<(bool, Epoch, f64)>>>> =
        HashMap::new();

    for ((epoch, _flag), (clock_offset, vehicles)) in record {
        if let Some(value) = clock_offset {
            clk_offset.push((*epoch, *value));
        }

        for (sv, observations) in vehicles {
            for (observable, data) in observations {
                let observable_code = observable.to_string();
                let physics = observable_to_physics(observable);
                let y = data.obs;
                let cycle_slip = match data.lli {
                    Some(lli) => lli.intersects(LliFlags::LOCK_LOSS),
                    _ => false,
                };

                if let Some(data) = dataset.get_mut(&physics) {
                    if let Some(data) = data.get_mut(&observable_code) {
                        if let Some(data) = data.get_mut(sv) {
                            data.push((cycle_slip, *epoch, y));
                        } else {
                            data.insert(*sv, vec![(cycle_slip, *epoch, y)]);
                        }
                    } else {
                        let mut map: HashMap<SV, Vec<(bool, Epoch, f64)>> = HashMap::new();
                        map.insert(*sv, vec![(cycle_slip, *epoch, y)]);
                        data.insert(observable_code, map);
                    }
                } else {
                    let mut map: HashMap<SV, Vec<(bool, Epoch, f64)>> = HashMap::new();
                    map.insert(*sv, vec![(cycle_slip, *epoch, y)]);
                    let mut mmap: HashMap<String, HashMap<SV, Vec<(bool, Epoch, f64)>>> =
                        HashMap::new();
                    mmap.insert(observable_code, map);
                    dataset.insert(physics.to_string(), mmap);
                }
            }
        }
    }

    if !clk_offset.is_empty() {
        plot_context.add_timedomain_plot("Receiver Clock Offset", "Clock Offset [s]");
        let data_x: Vec<Epoch> = clk_offset.iter().map(|(k, _)| *k).collect();
        let data_y: Vec<f64> = clk_offset.iter().map(|(_, v)| *v).collect();
        let trace = build_chart_epoch_axis(
            "Clk Offset",
            Mode::LinesMarkers,
            data_x.clone(),
            data_y.clone(),
        )
        .marker(Marker::new().symbol(MarkerSymbol::TriangleUp));
        plot_context.add_trace(trace);

        if csv_export {
            let fullpath = ctx.workspace.join("CSV").join("clock-offset.csv");

            let title = match header.rcvr.as_ref() {
                Some(rcvr) => {
                    format!("{} (#{}) Clock Offset", rcvr.model, rcvr.sn)
                },
                _ => "Receiver Clock Offset".to_string(),
            };
            csv_export_timedomain(
                &fullpath,
                &title,
                "Epoch, Clock Offset [s]",
                &data_x,
                &data_y,
            )
            .expect("failed to render data as CSV");
        }

        trace!("receiver clock offsets");
    }
    /*
     * 1 plot per physics
     */
    for (physics, carriers) in dataset {
        let y_label = match physics.as_str() {
            "Phase" => "Carrier cycles",
            "Doppler" => "Doppler Shifts",
            "Signal Strength" => "Power [dB]",
            "Pseudo Range" => "Pseudo Range",
            _ => unreachable!(),
        };

        if ctx.data.has_brdc_navigation() && ctx.data.has_sp3() {
            // Augmented context, we plot data on two Y axes
            // one for physical observation, one for sat elevation
            plot_context.add_timedomain_2y_plot(
                &format!("{} Observations", physics),
                y_label,
                "Elevation Angle [°]",
            );
        } else {
            // standard mode: one axis
            plot_context.add_timedomain_plot(&format!("{} Observations", physics), y_label);
        }

        let markers = generate_markers(carriers.len()); // one symbol per carrier
        for (index, (observable, vehicles)) in carriers.iter().enumerate() {
            for (sv_index, (sv, data)) in vehicles.iter().enumerate() {
                let data_x: Vec<Epoch> = data.iter().map(|(_cs, e, _y)| *e).collect();
                let data_y: Vec<f64> = data.iter().map(|(_cs, _e, y)| *y).collect();

                let trace = build_chart_epoch_axis(
                    &format!("{:X}({})", sv, observable),
                    Mode::Markers,
                    data_x.clone(),
                    data_y.clone(),
                )
                .marker(Marker::new().symbol(markers[index].clone()))
                .visible({
                    if sv_index == 0 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                });
                plot_context.add_trace(trace);

                if csv_export {
                    let fullpath = ctx
                        .workspace
                        .join("CSV")
                        .join(&format!("{}-{}.csv", sv, observable));
                    csv_export_timedomain(
                        &fullpath,
                        &format!("{} observations", observable),
                        "Epoch, Observation",
                        &data_x.clone(),
                        &data_y.clone(),
                    )
                    .expect("failed to render data as CSV");
                }

                if index == 0 && physics == "Signal Strength" {
                    // Draw SV elevation along SSI plot if that is feasible
                    if let Some(nav) = ctx.data.brdc_navigation() {
                        // determine SV state
                        let rx_ecef = ctx.rx_ecef.unwrap();
                        let data = data_x
                            .iter()
                            .filter_map(|t| {
                                nav.sv_position_interpolate(*sv, *t, 5)
                                    .map(|(x_km, y_km, z_km)| {
                                        (
                                            *t,
                                            Ephemeris::elevation_azimuth(
                                                (x_km * 1.0E3, y_km * 1.0E3, z_km * 1.0E3),
                                                rx_ecef,
                                            )
                                            .0,
                                        )
                                    })
                            })
                            .collect::<Vec<_>>();
                        // plot
                        let data_x = data.iter().map(|(x, _)| *x).collect::<Vec<_>>();
                        let data_y = data.iter().map(|(_, y)| *y).collect::<Vec<_>>();
                        let trace = build_chart_epoch_axis(
                            &format!("BRDC_Elev({:X})", sv),
                            Mode::Markers,
                            data_x,
                            data_y,
                        )
                        .y_axis("y2")
                        .marker(Marker::new().symbol(markers[index].clone()))
                        .visible({
                            if sv_index == 0 && index == 0 {
                                Visible::True
                            } else {
                                Visible::LegendOnly
                            }
                        });
                        plot_context.add_trace(trace);
                    }
                    if let Some(sp3) = ctx.data.sp3() {
                        // determine SV state
                        let rx_ecef = ctx.rx_ecef.unwrap();
                        let data = data_x
                            .iter()
                            .filter_map(|t| {
                                sp3.sv_position_interpolate(*sv, *t, 5)
                                    .map(|(x_km, y_km, z_km)| {
                                        (
                                            *t,
                                            Ephemeris::elevation_azimuth(
                                                (x_km * 1.0E3, y_km * 1.0E3, z_km * 1.0E3),
                                                rx_ecef,
                                            )
                                            .0,
                                        )
                                    })
                            })
                            .collect::<Vec<_>>();
                        // plot
                        let data_x = data.iter().map(|(x, _)| *x).collect::<Vec<_>>();
                        let data_y = data.iter().map(|(_, y)| *y).collect::<Vec<_>>();
                        let trace = build_chart_epoch_axis(
                            &format!("SP3_Elev({:X})", sv),
                            Mode::Markers,
                            data_x,
                            data_y,
                        )
                        .y_axis("y2")
                        .marker(Marker::new().symbol(markers[index].clone()))
                        .visible({
                            if sv_index == 0 && index == 0 {
                                Visible::True
                            } else {
                                Visible::LegendOnly
                            }
                        });
                        plot_context.add_trace(trace);
                    }
                }
            }
        }
        trace!("{} observations", y_label);
    }
}
