use crate::{
    plot::{build_chart_epoch_axis, generate_markers, PlotContext},
    Context,
};
use plotly::common::{Marker, MarkerSymbol, Mode, Visible};
use rinex::{observation::*, prelude::*};
use std::collections::HashMap;

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
pub fn plot_observation(ctx: &Context, plot_ctx: &mut PlotContext) {
    let record = ctx.primary_rinex.record.as_obs().unwrap();

    let mut clk_offset: Vec<(Epoch, f64)> = Vec::new();
    // dataset
    //  per physics,
    //   per observable (symbolized)
    //      per vehicle (color map)
    //      bool: loss of lock - CS emphasis
    //      x: sampling timestamp,
    //      y: observation (raw),
    let mut dataset: HashMap<String, HashMap<String, HashMap<Sv, Vec<(bool, Epoch, f64)>>>> =
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
                        if let Some(data) = data.get_mut(&sv) {
                            data.push((cycle_slip, *epoch, y));
                        } else {
                            data.insert(*sv, vec![(cycle_slip, *epoch, y)]);
                        }
                    } else {
                        let mut map: HashMap<Sv, Vec<(bool, Epoch, f64)>> = HashMap::new();
                        map.insert(*sv, vec![(cycle_slip, *epoch, y)]);
                        data.insert(observable_code, map);
                    }
                } else {
                    let mut map: HashMap<Sv, Vec<(bool, Epoch, f64)>> = HashMap::new();
                    map.insert(*sv, vec![(cycle_slip, *epoch, y)]);
                    let mut mmap: HashMap<String, HashMap<Sv, Vec<(bool, Epoch, f64)>>> =
                        HashMap::new();
                    mmap.insert(observable_code, map);
                    dataset.insert(physics.to_string(), mmap);
                }
            }
        }
    }

    if clk_offset.len() > 0 {
        plot_ctx.add_cartesian2d_plot("Receiver Clock Offset", "Clock Offset [s]");
        let data_x: Vec<Epoch> = clk_offset.iter().map(|(k, _)| *k).collect();
        let data_y: Vec<f64> = clk_offset.iter().map(|(_, v)| *v).collect();
        let trace = build_chart_epoch_axis("Clk Offset", Mode::LinesMarkers, data_x, data_y)
            .marker(Marker::new().symbol(MarkerSymbol::TriangleUp));
        plot_ctx.add_trace(trace);
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

        if let Some(_) = ctx.nav_rinex {
            // Augmented context, we plot data on two Y axes
            // one for physical observation, one for sat elevation
            plot_ctx.add_cartesian2d_2y_plot(
                &format!("{} Observations", physics),
                y_label,
                "Elevation Angle [°]",
            );
        } else {
            // standard mode: one axis
            plot_ctx.add_cartesian2d_plot(&format!("{} Observations", physics), y_label);
        }

        let markers = generate_markers(carriers.len()); // one symbol per carrier
        for (index, (observable, vehicles)) in carriers.iter().enumerate() {
            for (sv, data) in vehicles {
                let data_x: Vec<Epoch> = data.iter().map(|(_cs, e, _y)| *e).collect();
                let data_y: Vec<f64> = data.iter().map(|(_cs, _e, y)| *y).collect();

                let trace = build_chart_epoch_axis(
                    &format!("{}({})", sv, observable),
                    Mode::Markers,
                    data_x,
                    data_y,
                )
                .marker(Marker::new().symbol(markers[index].clone()))
                .web_gl_mode(true)
                .visible({
                    if index < 1 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                });
                plot_ctx.add_trace(trace);

                if index == 0 && physics == "Signal Strength" {
                    // 1st Carrier encountered: plot Sv only once
                    // we also only augment the SSI plot when NAV context is provided
                    if let Some(ref nav) = ctx.nav_rinex {
                        // grab elevation angle
                        let data: Vec<(Epoch, f64)> = nav
                            .sv_elevation_azimuth(ctx.ground_position)
                            .map(|(epoch, (_sv, (elev, _a)))| (epoch, elev))
                            .collect();
                        // plot (Epoch, Elev)
                        let epochs: Vec<Epoch> = data.iter().map(|(e, _)| *e).collect();
                        let elev: Vec<f64> = data.iter().map(|(_, f)| *f).collect();
                        let trace = build_chart_epoch_axis(
                            &format!("Elev({})", sv),
                            Mode::LinesMarkers,
                            epochs,
                            elev,
                        )
                        .marker(Marker::new().symbol(markers[index].clone()))
                        .visible(Visible::LegendOnly);
                        plot_ctx.add_trace(trace);
                    }
                }
            }
        }
        trace!("{} observations", y_label);
    }
}
