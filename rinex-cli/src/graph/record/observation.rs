use crate::cli::Context;
use plotly::{
    color::NamedColor,
    common::{Marker, MarkerSymbol, Mode, Visible},
};
use std::collections::HashMap;

use rinex::{navigation::Ephemeris, prelude::*};

use crate::graph::{build_chart_epoch_axis, csv_export_timedomain, generate_markers, PlotContext};

#[derive(Debug, PartialEq, Eq, Hash)]
enum Physics {
    SSI,
    Doppler,
    Phase,
    PseudoRange,
}

impl Physics {
    fn from_observable(observable: &Observable) -> Self {
        if observable.is_phase_observable() {
            Self::Phase
        } else if observable.is_doppler_observable() {
            Self::Doppler
        } else if observable.is_ssi_observable() {
            Self::SSI
        } else {
            Self::PseudoRange
        }
    }
    fn plot_title(&self) -> String {
        match self {
            Self::SSI => "SSI".to_string(),
            Self::Phase => "Phase".to_string(),
            Self::Doppler => "Doppler".to_string(),
            Self::PseudoRange => "Pseudo Range".to_string(),
        }
    }
    fn y_axis(&self) -> String {
        match self {
            Self::SSI => "Power [dB]".to_string(),
            Self::Phase => "Carrier Cycles".to_string(),
            Self::Doppler => "Doppler Shifts".to_string(),
            Self::PseudoRange => "Pseudo Range".to_string(),
        }
    }
}

/*
 * Plots given Observation RINEX content
 */
pub fn plot_observations(ctx: &Context, plot_ctx: &mut PlotContext, csv_export: bool) {
    let obs_data = ctx.data.observation().unwrap(); // infaillible
    let header = &obs_data.header;
    let record = obs_data.record.as_obs().unwrap(); // infaillible

    /////////////////////////////////////////////////////
    // Gather all data, complex type but single iteration..
    // RX OK or ERROR
    //  per physics,
    //   per observable (symbolized)
    //      per vehicle (color map)
    //      x: sampling timestamp,
    //      y: observation (raw),
    /////////////////////////////////////////////////////
    let mut clk_offset_good: Vec<(Epoch, f64)> = Vec::with_capacity(64);
    let mut clk_offset_bad: Vec<(Epoch, f64)> = Vec::with_capacity(64);
    let mut dataset_good: HashMap<Physics, HashMap<String, HashMap<SV, Vec<(Epoch, f64)>>>> =
        HashMap::with_capacity(1024);
    let mut dataset_bad: HashMap<Physics, HashMap<String, HashMap<SV, Vec<(Epoch, f64)>>>> =
        HashMap::with_capacity(1024);

    for ((epoch, flag), (clock_offset, vehicles)) in record {
        if flag.is_ok() {
            if let Some(value) = clock_offset {
                clk_offset_good.push((*epoch, *value));
            }
            for (sv, observations) in vehicles {
                for (observable, data) in observations {
                    let observable_code = observable.to_string();
                    let physics = Physics::from_observable(observable);
                    let y = data.obs;

                    if let Some(data) = dataset_good.get_mut(&physics) {
                        if let Some(data) = data.get_mut(&observable_code) {
                            if let Some(data) = data.get_mut(sv) {
                                data.push((*epoch, y));
                            } else {
                                data.insert(*sv, vec![(*epoch, y)]);
                            }
                        } else {
                            let mut map: HashMap<SV, Vec<(Epoch, f64)>> = HashMap::new();
                            map.insert(*sv, vec![(*epoch, y)]);
                            data.insert(observable_code, map);
                        }
                    } else {
                        let mut map: HashMap<SV, Vec<(Epoch, f64)>> = HashMap::new();
                        map.insert(*sv, vec![(*epoch, y)]);
                        let mut mmap: HashMap<String, HashMap<SV, Vec<(Epoch, f64)>>> =
                            HashMap::new();
                        mmap.insert(observable_code, map);
                        dataset_good.insert(physics, mmap);
                    }
                }
            }
        } else {
            if let Some(value) = clock_offset {
                clk_offset_bad.push((*epoch, *value));
            }
            for (sv, observations) in vehicles {
                for (observable, data) in observations {
                    let observable_code = observable.to_string();
                    let physics = Physics::from_observable(observable);
                    let y = data.obs;

                    if let Some(data) = dataset_bad.get_mut(&physics) {
                        if let Some(data) = data.get_mut(&observable_code) {
                            if let Some(data) = data.get_mut(sv) {
                                data.push((*epoch, y));
                            } else {
                                data.insert(*sv, vec![(*epoch, y)]);
                            }
                        } else {
                            let mut map: HashMap<SV, Vec<(Epoch, f64)>> = HashMap::new();
                            map.insert(*sv, vec![(*epoch, y)]);
                            data.insert(observable_code, map);
                        }
                    } else {
                        let mut map: HashMap<SV, Vec<(Epoch, f64)>> = HashMap::new();
                        map.insert(*sv, vec![(*epoch, y)]);
                        let mut mmap: HashMap<String, HashMap<SV, Vec<(Epoch, f64)>>> =
                            HashMap::new();
                        mmap.insert(observable_code, map);
                        dataset_bad.insert(physics, mmap);
                    }
                }
            }
        }
    }

    /////////////////////////////
    // Plot Clock offset (if any)
    /////////////////////////////
    if !clk_offset_good.is_empty() || !clk_offset_bad.is_empty() {
        plot_ctx.add_timedomain_plot("Receiver Clock Offset", "Clock Offset [s]");
        let good_x: Vec<Epoch> = clk_offset_good.iter().map(|(x, _)| *x).collect();
        let good_y: Vec<f64> = clk_offset_good.iter().map(|(_, y)| *y).collect();

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
                &good_x,
                &good_y,
            )
            .expect("failed to render data as CSV");
        }

        let trace = build_chart_epoch_axis("Clk Offset", Mode::LinesMarkers, good_x, good_y)
            .marker(Marker::new().symbol(MarkerSymbol::TriangleUp));
        plot_ctx.add_trace(trace);
        trace!("receiver clock offsets");
    }

    ////////////////////////////////
    // Generate 1 plot per physics
    ////////////////////////////////
    for physics in [Physics::PseudoRange, Physics::Phase, Physics::Doppler] {
        if let Some(observables) = dataset_good.get(&physics) {
            let title = physics.plot_title();
            let y_label = physics.y_axis();
            plot_ctx.add_timedomain_plot(&title, &y_label);

            let markers = generate_markers(observables.len());
            for (index, (observable, vehicles)) in observables.iter().enumerate() {
                for (sv_index, (sv, data)) in vehicles.iter().enumerate() {
                    let good_x: Vec<_> = data.iter().map(|(x, _y)| *x).collect::<_>();
                    let good_y: Vec<_> = data.iter().map(|(_x, y)| *y).collect::<_>();

                    if csv_export {
                        let fullpath = ctx
                            .workspace
                            .join("CSV")
                            .join(&format!("{}-{}.csv", sv, observable));
                        csv_export_timedomain(
                            &fullpath,
                            &format!("{} observations", observable),
                            "Epoch, Observation",
                            &good_x,
                            &good_y,
                        )
                        .expect("failed to render data as CSV");
                    }

                    let trace = build_chart_epoch_axis(
                        &format!("{:X}({})", sv, observable),
                        Mode::Markers,
                        good_x,
                        good_y,
                    )
                    .marker(Marker::new().symbol(markers[index].clone()))
                    .visible({
                        if index == 0 && sv_index == 0 {
                            Visible::True
                        } else {
                            Visible::LegendOnly
                        }
                    });
                    plot_ctx.add_trace(trace);
                }
            }
            if let Some(bad_observables) = dataset_bad.get(&physics) {
                for (index, (bad_observable, bad_sv)) in bad_observables.iter().enumerate() {
                    for (sv_index, (sv, data)) in bad_sv.iter().enumerate() {
                        let bad_x: Vec<_> = data.iter().map(|(x, _y)| *x).collect::<_>();
                        let bad_y: Vec<_> = data.iter().map(|(_x, y)| *y).collect::<_>();
                        let trace = build_chart_epoch_axis(
                            &format!("{:X}({})", sv, bad_observable),
                            Mode::Markers,
                            bad_x,
                            bad_y,
                        )
                        .marker(
                            Marker::new()
                                .symbol(markers[index].clone())
                                .color(NamedColor::Black),
                        )
                        .visible({
                            if index == 0 && sv_index == 0 {
                                Visible::True
                            } else {
                                Visible::LegendOnly
                            }
                        });
                        plot_ctx.add_trace(trace);
                    }
                }
            }
            trace!("{} plot", title);
        }
    }

    ////////////////////////////////////////////
    // Generate 1 plot for SSI
    // that we possibly augment with NAV context
    ////////////////////////////////////////////
    if let Some(good_observables) = dataset_good.get(&Physics::SSI) {
        let title = Physics::SSI.plot_title();
        let y_label = Physics::SSI.y_axis();

        let augmented = ctx.data.has_brdc_navigation() || ctx.data.has_sp3();

        if augmented {
            plot_ctx.add_timedomain_2y_plot(&title, &y_label, "Elevation [Degrees]");
        } else {
            plot_ctx.add_timedomain_plot(&title, &y_label);
        }

        // Plot Observations
        let markers = generate_markers(good_observables.len());
        for (index, (observable, vehicles)) in good_observables.iter().enumerate() {
            for (sv_index, (sv, data)) in vehicles.iter().enumerate() {
                let good_x: Vec<_> = data.iter().map(|(x, _y)| *x).collect::<_>();
                let good_y: Vec<_> = data.iter().map(|(_x, y)| *y).collect::<_>();

                if csv_export {
                    let fullpath = ctx
                        .workspace
                        .join("CSV")
                        .join(&format!("{}-{}.csv", sv, observable));
                    csv_export_timedomain(
                        &fullpath,
                        &format!("{} observations", observable),
                        "Epoch, Observation",
                        &good_x,
                        &good_y,
                    )
                    .expect("failed to render data as CSV");
                }

                // Augment (if possible)
                if augmented && index == 0 {
                    // determine SV state
                    let rx_ecef = ctx.rx_ecef.unwrap();

                    if let Some(nav) = ctx.data.brdc_navigation() {
                        let data = good_x
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
                        plot_ctx.add_trace(trace);
                    }
                    if let Some(sp3) = ctx.data.sp3() {
                        let data = good_x
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
                        plot_ctx.add_trace(trace);
                    }
                }

                let trace = build_chart_epoch_axis(
                    &format!("{:X}({})", sv, observable),
                    Mode::Markers,
                    good_x,
                    good_y,
                )
                .marker(Marker::new().symbol(markers[index].clone()))
                .y_axis("y1")
                .visible({
                    if index == 0 && sv_index == 0 {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                });
                plot_ctx.add_trace(trace);
            }
        }
        trace!("{} observations", y_label);
    }
}
