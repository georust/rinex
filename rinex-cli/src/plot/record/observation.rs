use crate::{
    plot::{build_chart_epoch_axis, generate_markers, PlotContext},
    Context,
};
use plotly::common::{Marker, MarkerSymbol, Mode, Visible};
use rinex::{observation::*, prelude::*};
use std::collections::{BTreeMap, HashMap};

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
    //  per physics, per carrier signal (symbol)
    //      per vehicule (color map)
    //      bool: loss of lock - CS emphasis
    //      x: sampling timestamp,
    //      y: observation (raw),
    let mut dataset: HashMap<String, HashMap<u8, HashMap<Sv, Vec<(bool, Epoch, f64)>>>> =
        HashMap::new();

    // augmented mode
    let mut sat_angles: HashMap<Sv, BTreeMap<Epoch, (f64, f64)>> = HashMap::new();
    if let Some(ref nav) = ctx.nav_rinex {
        sat_angles = nav.navigation_sat_angles(ctx.ground_position);
    }

    for ((epoch, _flag), (clock_offset, vehicules)) in record {
        if let Some(value) = clock_offset {
            clk_offset.push((*epoch, *value));
        }

        for (sv, observations) in vehicules {
            for (observable, data) in observations {
                let code = observable.to_string();
                let carrier_code = &code[1..2]; // carrier code
                let c_code =
                    u8::from_str_radix(carrier_code, 10).expect("failed to parse carrier code");

                let physics = observable_to_physics(observable);
                let y = data.obs;
                let cycle_slip = match data.lli {
                    Some(lli) => lli.intersects(LliFlags::LOCK_LOSS),
                    _ => false,
                };

                if let Some(data) = dataset.get_mut(&physics) {
                    if let Some(data) = data.get_mut(&c_code) {
                        if let Some(data) = data.get_mut(&sv) {
                            data.push((cycle_slip, *epoch, y));
                        } else {
                            data.insert(*sv, vec![(cycle_slip, *epoch, y)]);
                        }
                    } else {
                        let mut map: HashMap<Sv, Vec<(bool, Epoch, f64)>> = HashMap::new();
                        map.insert(*sv, vec![(cycle_slip, *epoch, y)]);
                        data.insert(c_code, map);
                    }
                } else {
                    let mut map: HashMap<Sv, Vec<(bool, Epoch, f64)>> = HashMap::new();
                    map.insert(*sv, vec![(cycle_slip, *epoch, y)]);
                    let mut mmap: HashMap<u8, HashMap<Sv, Vec<(bool, Epoch, f64)>>> =
                        HashMap::new();
                    mmap.insert(c_code, map);
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

        if sat_angles.len() > 0 {
            plot_ctx.add_cartesian2d_2y_plot(
                &format!("{} Observations", physics),
                y_label,
                "Elevation Angle [°]",
            );
        } else {
            plot_ctx.add_cartesian2d_plot(&format!("{} Observations", physics), y_label);
        }

        let markers = generate_markers(carriers.len()); // one symbol per carrier
        for (index, (carrier, vehicules)) in carriers.iter().enumerate() {
            for (sv, data) in vehicules {
                let data_x: Vec<Epoch> = data.iter().map(|(_cs, e, _y)| *e).collect();
                let data_y: Vec<f64> = data.iter().map(|(_cs, _e, y)| *y).collect();
                let trace = build_chart_epoch_axis(
                    &format!("{}(L{})", sv, carrier),
                    Mode::Markers,
                    data_x,
                    data_y,
                )
                .marker(Marker::new().symbol(markers[index].clone()))
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
                    // we also only augment the SSI plot
                    if let Some(epochs) = sat_angles.get(sv) {
                        let elev: Vec<f64> = epochs.iter().map(|(_, (el, _azi))| *el).collect();
                        let epochs: Vec<Epoch> = epochs.keys().map(|k| *k).collect();
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
