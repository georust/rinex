use crate::plot::{generate_markers, Context};
use plotly::{
    common::{Marker, MarkerSymbol, Mode},
    Scatter,
};
use rinex::{navigation::*, observation::*, prelude::*, *};
use std::collections::HashMap;

macro_rules! code2physics {
    ($code: expr) => {
        if is_phase_carrier_obs_code!($code) {
            "Phase".to_string()
        } else if is_doppler_obs_code!($code) {
            "Doppler".to_string()
        } else if is_sig_strength_obs_code!($code) {
            "Signal Strength".to_string()
        } else {
            "Pseudo Range".to_string()
        }
    };
}

/*
 * Plots given Observation RINEX content
 */
pub fn plot_observation(ctx: &mut Context, record: &observation::Record, nav_ctx: &Option<Rinex>) {
    if let Some(nav) = nav_ctx {
        //enhanced_plot(record, nav);
        basic_plot(ctx, record);
    } else {
        basic_plot(ctx, record);
    }
}

pub fn basic_plot(ctx: &mut Context, record: &observation::Record) {
    let mut clk_offset: Vec<(String, f64)> = Vec::new();
    // dataset
    //  per physics, per carrier signal (symbol)
    //      per vehicule (color map)
    //      bool: loss of lock - CS emphasis
    //      x: sampling timestamp,
    //      y: observation (raw),
    let mut dataset: HashMap<String, HashMap<u8, HashMap<Sv, Vec<(bool, String, f64)>>>> =
        HashMap::new();
    for ((epoch, _flag), (clock_offset, vehicules)) in record {
        if let Some(value) = clock_offset {
            clk_offset.push((epoch.to_string(), *value));
        }

        for (sv, observations) in vehicules {
            for (observation, data) in observations {
                let p_code = &observation[0..1];
                let c_code = &observation[1..2]; // carrier code
                let c_code = u8::from_str_radix(c_code, 10).expect("failed to parse carrier code");

                let physics = code2physics!(p_code);
                let y = data.obs;
                let cycle_slip = match data.lli {
                    Some(lli) => lli.intersects(LliFlags::LOCK_LOSS),
                    _ => false,
                };

                if let Some(data) = dataset.get_mut(&physics) {
                    if let Some(data) = data.get_mut(&c_code) {
                        if let Some(data) = data.get_mut(&sv) {
                            data.push((cycle_slip, epoch.to_string(), y));
                        } else {
                            data.insert(*sv, vec![(cycle_slip, epoch.to_string(), y)]);
                        }
                    } else {
                        let mut map: HashMap<Sv, Vec<(bool, String, f64)>> = HashMap::new();
                        map.insert(*sv, vec![(cycle_slip, epoch.to_string(), y)]);
                        data.insert(c_code, map);
                    }
                } else {
                    let mut map: HashMap<Sv, Vec<(bool, String, f64)>> = HashMap::new();
                    map.insert(*sv, vec![(cycle_slip, epoch.to_string(), y)]);
                    let mut mmap: HashMap<u8, HashMap<Sv, Vec<(bool, String, f64)>>> =
                        HashMap::new();
                    mmap.insert(c_code, map);
                    dataset.insert(physics.to_string(), mmap);
                }
            }
        }
    }

    if clk_offset.len() > 0 {
        ctx.add_cartesian2d_plot("Receiver Clock Offset", "Clock Offset [s]");
        let data_x: Vec<String> = clk_offset.iter().map(|(k, _)| k.clone()).collect();
        let data_y: Vec<f64> = clk_offset.iter().map(|(_, v)| *v).collect();
        let trace = Scatter::new(data_x, data_y)
            .mode(Mode::LinesMarkers)
            .marker(Marker::new().symbol(MarkerSymbol::TriangleUp))
            .name("Clk Offset");
        ctx.add_trace(trace);
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
        ctx.add_cartesian2d_plot(&format!("{} Observations", physics), y_label);
        // one symbol per carrier
        let markers = generate_markers(carriers.len());
        for (index, (carrier, vehicules)) in carriers.iter().enumerate() {
            for (sv, data) in vehicules {
                let data_x: Vec<String> = data.iter().map(|(cs, e, _y)| e.clone()).collect();
                let data_y: Vec<f64> = data.iter().map(|(cs, _e, y)| *y).collect();
                let trace = Scatter::new(data_x, data_y)
                    .mode(Mode::Markers)
                    .marker(Marker::new().symbol(markers[index].clone()))
                    .name(&format!("{}(L{})", sv, carrier));
                ctx.add_trace(trace);
            }
        }
    }
}

/*
pub fn enhanced_plot(ctx: &mut Context, record: &observation::Record, nav: &Rinex) {
    let mut e0: f64 = 0.0;
    let cmap = colorous::TURBO; // to differentiate vehicules (PRN#)
    let symbols = vec!["x", "t", "o"]; // to differentiate carrier signals
                                       // sorted by Physics, By Carrier number, By vehicule, (x,y)
    let mut clk_offset: Vec<(f64, f64)> = Vec::new();
    // dataset
    //  per physics, per carrier signal (symbol)
    //      per vehicule (color map)
    //      x: sampling timestamp,
    //      y: observation (raw),
    //      bool: loss of lock - CS emphasis
    //      optionnal(f64): Sv elevation angle, if NAV is provided
    let mut dataset: HashMap<String, HashMap<u8, HashMap<Sv, Vec<(bool, f64, f64)>>>> =
        HashMap::new();

    let mut elev_angles: HashMap<Sv, Vec<(f64, f64)>> = HashMap::new();
    let nav_rec = nav
        .record
        .as_nav()
        .expect("`--nav` should be navigation data");
    let rcvr_pos = nav
        .header
        .coords
        .expect("--nav should contain receiver position");

    for (e_index, ((epoch, _flag), (clock_offset, vehicules))) in record.iter().enumerate() {
        if e_index == 0 {
            e0 = epoch.to_utc_seconds();
        }

        let e = epoch.to_utc_seconds();
        let x = e - e0;
        if let Some(value) = clock_offset {
            clk_offset.push((x, *value));
        }

        // retrieve related epoch in Nav
        let nav_classes = nav_rec.get(&epoch);
        if let Some(classes) = nav_classes {
            println!("{}: {}", epoch, classes.len());
        } else {
            println!("{}: NONE", epoch);
        }

        for (sv, observations) in vehicules {
            // retrieve related elevation angle
            if let Some(classes) = nav_classes {
                for (class, frames) in classes {
                    if *class == FrameClass::Ephemeris {
                        for fr in frames {
                            let (_, nav_sv, eph) = fr.as_eph().unwrap();
                            if nav_sv == sv {
                                if let Some((e, _)) = eph.sat_angles(*epoch, rcvr_pos) {
                                    if let Some(data) = elev_angles.get_mut(sv) {
                                        data.push((x, e));
                                    } else {
                                        elev_angles.insert(*sv, vec![(x, e)]);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            for (observation, data) in observations {
                let p_code = &observation[0..1];
                let c_code = &observation[1..2]; // carrier code
                let c_code = u8::from_str_radix(c_code, 10).expect("failed to parse carrier code");

                let physics = code2physics!(p_code);
                let y = data.obs;
                let cycle_slip = match data.lli {
                    Some(lli) => lli.intersects(LliFlags::LOCK_LOSS),
                    _ => false,
                };

                if let Some(data) = dataset.get_mut(&physics) {
                    if let Some(data) = data.get_mut(&c_code) {
                        if let Some(data) = data.get_mut(&sv) {
                            data.push((cycle_slip, x, y));
                        } else {
                            data.insert(*sv, vec![(cycle_slip, x, y)]);
                        }
                    } else {
                        let mut map: HashMap<Sv, Vec<(bool, f64, f64)>> = HashMap::new();
                        map.insert(*sv, vec![(cycle_slip, x, y)]);
                        data.insert(c_code, map);
                    }
                } else {
                    let mut map: HashMap<Sv, Vec<(bool, f64, f64)>> = HashMap::new();
                    map.insert(*sv, vec![(cycle_slip, x, y)]);
                    let mut mmap: HashMap<u8, HashMap<Sv, Vec<(bool, f64, f64)>>> = HashMap::new();
                    mmap.insert(c_code, map);
                    dataset.insert(physics.to_string(), mmap);
                }
            }
        }
    }

    if let Some(plot) = ctx.plots.get("clock-offset.png") {
        let mut chart = ctx
            .charts
            .get("Clock Offset")
            .expect("faulty plot context: no chart defined for clock offsets")
            .clone()
            .restore(plot);
        chart
            .draw_series(clk_offset.iter().map(|point| {
                TriangleMarker::new(*point, 4, Into::<ShapeStyle>::into(&BLACK).filled()).into_dyn()
            }))
            .expect("failed to plot receiver clock offsets");
        chart
            .draw_series(LineSeries::new(
                clk_offset.iter().map(|point| *point),
                &BLACK,
            ))
            .expect("failed to plot receiver clock offsets")
            .label("Offset [s]")
            .legend(move |point| {
                TriangleMarker::new(point, 4, Into::<ShapeStyle>::into(&BLACK).filled()).into_dyn()
            });
        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(WHITE.filled())
            .draw()
            .expect("failed to draw clock offset labels");
    }
    /*
     * 1 plot per physics
     */
    for (physics, carriers) in dataset {
        // retrieve dedicated plot
        let plot_title: &str = match physics.as_str() {
            "Phase" => "phase.png",
            "Doppler" => "doppler.png",
            "Signal Strength" => "ssi.png",
            "Pseudo Range" => "pseudo-range.png",
            _ => unreachable!(),
        };
        let plot = ctx.plots.get(plot_title).expect(&format!(
            "faulty context: missing plot for \"{}\" data",
            physics
        ));
        // retrieve associated chart
        let mut chart = ctx
            .dual_charts
            .get(&physics)
            .expect(&format!("faulty context: missing \"{}\" chart", physics))
            .clone()
            .restore(plot);
        /*
         * plot this kind of data
         */
        for (carrier, vehicules) in carriers {
            let symbol = symbols[carrier as usize % symbols.len()]; // one symbol per carrier
            for (sv, data) in vehicules {
                // retrieve color from color map
                let color = cmap.eval_rational(sv.prn.into(), 50);
                let color = RGBColor {
                    0: color.r,
                    1: color.g,
                    2: color.b,
                };
                // plot
                chart
                    .draw_series(data.iter().map(|(cycle_slip, x, y)| {
                        if *cycle_slip {
                            match symbol {
                                "x" => Cross::new(
                                    (*x, *y),
                                    5,
                                    Into::<ShapeStyle>::into(&BLACK).filled().stroke_width(2),
                                )
                                .into_dyn(),
                                "o" => Circle::new(
                                    (*x, *y),
                                    5,
                                    Into::<ShapeStyle>::into(&BLACK).filled().stroke_width(2),
                                )
                                .into_dyn(),
                                _ => TriangleMarker::new(
                                    (*x, *y),
                                    5,
                                    Into::<ShapeStyle>::into(&BLACK).filled().stroke_width(2),
                                )
                                .into_dyn(),
                            }
                        } else {
                            match symbol {
                                "x" => Cross::new(
                                    (*x, *y),
                                    4,
                                    Into::<ShapeStyle>::into(&color).filled(),
                                )
                                .into_dyn(),
                                "o" => Circle::new(
                                    (*x, *y),
                                    4,
                                    Into::<ShapeStyle>::into(&color).filled(),
                                )
                                .into_dyn(),
                                _ => TriangleMarker::new(
                                    (*x, *y),
                                    4,
                                    Into::<ShapeStyle>::into(&color).filled(),
                                )
                                .into_dyn(),
                            }
                        }
                    }))
                    .expect(&format!("failed to draw {} observations", physics))
                    .label(format!("{}(L{})", sv, carrier))
                    .legend(move |point| match symbol {
                        "x" => Cross::new(point, 4, Into::<ShapeStyle>::into(&color).filled())
                            .into_dyn(),
                        "o" => Circle::new(point, 4, Into::<ShapeStyle>::into(&color).filled())
                            .into_dyn(),
                        _ => {
                            TriangleMarker::new(point, 4, Into::<ShapeStyle>::into(&color).filled())
                                .into_dyn()
                        },
                    });

                /*
                 * possible plot enhancement
                 */
                if let Some(data) = elev_angles.get(&sv) {
                    // --> enhance this plot
                    chart
                        .draw_secondary_series(data.iter().map(|(x, y)| {
                            Circle::new((*x, *y), 5, Into::<ShapeStyle>::into(&RED).filled())
                                .into_dyn()
                        }))
                        .expect(&format!(
                            "failed to enhance {} plot with elevation angle",
                            physics
                        ))
                        .label(format!("{}", sv))
                        .legend(move |(x, y)| {
                            Circle::new((x + 10, y), 4, Into::<ShapeStyle>::into(&RED).filled())
                                .into_dyn()
                        });
                }
            }
        }
        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(WHITE.filled())
            .draw()
            .expect(&format!("failed to draw labels on {} chart", physics));
    }
}
*/
