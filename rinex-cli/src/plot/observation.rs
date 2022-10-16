//! Observation record plotting
use rinex::*;
use rinex::observation::Record;
use super::Context;
use plotters::prelude::*;
use std::collections::HashMap;

pub fn plot(ctx: &mut Context, record: &Record) {
    //TODO
    // emphasize LLI/SSI somehow ?
    let mut e0: i64 = 0;
    let mut clock_offsets: Vec<(f64, f64)> = Vec::new();
    let symbols = vec!["x","t","o"]; // to differentiate carrier signals
    let mut carrier: usize = 0;
    let mut pr: HashMap<usize, HashMap<Sv, Vec<(f64, f64)>>> = HashMap::new();
    let mut ssi: HashMap<usize, HashMap<Sv, Vec<(f64,f64)>>> = HashMap::new();
    let mut phase: HashMap<usize, HashMap<Sv, Vec<(f64,f64)>>> = HashMap::new();
    let mut doppler: HashMap<usize, HashMap<Sv, Vec<(f64,f64)>>> = HashMap::new();
    for (index, (epoch, (clock_offset, vehicules))) in record.iter().enumerate() {
        if index == 0 {
            e0 = epoch.date.timestamp()
        }
        let e = epoch.date.timestamp();
        if let Some(value) = clock_offset {
            clock_offsets.push(((e-e0) as f64, *value));
        }
        for (vehicule, observations) in vehicules {
            for (observation, data) in observations {
                if observation.contains("1") {
                    carrier = 0;
                } else if observation.contains("2") {
                    carrier = 1;
                } else {
                    carrier = 2;
                }
                if is_phase_carrier_obs_code!(observation) {
                    if let Some(vehicules) = phase.get_mut(&carrier) {
                        if let Some(phases) = vehicules.get_mut(vehicule) {
                            phases.push(((e-e0) as f64, data.obs));
                        } else {
                            vehicules.insert(*vehicule, vec![((e-e0) as f64, data.obs)]);
                        }
                    } else {
                        let mut map: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
                        map.insert(*vehicule, vec![((e-e0) as f64, data.obs)]);
                        phase.insert(carrier, map);
                    }
                } else if is_doppler_obs_code!(observation) {
                    if let Some(vehicules) = doppler.get_mut(&carrier) {
                        if let Some(doppler) = vehicules.get_mut(vehicule) {
                            doppler.push(((e-e0) as f64, data.obs));
                        } else {
                            vehicules.insert(*vehicule, vec![((e-e0) as f64, data.obs)]);
                        }
                    } else {
                        let mut map: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
                        map.insert(*vehicule, vec![((e-e0) as f64, data.obs)]);
                        doppler.insert(carrier, map);
                    }
                } else if is_pseudo_range_obs_code!(observation) {
                    if let Some(vehicules) = pr.get_mut(&carrier) {
                        if let Some(pr) = vehicules.get_mut(vehicule) {
                            pr.push(((e-e0) as f64, data.obs));
                        } else {
                            vehicules.insert(*vehicule, vec![((e-e0) as f64, data.obs)]);
                        }
                    } else {
                        let mut map: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
                        map.insert(*vehicule, vec![((e-e0) as f64, data.obs)]);
                        pr.insert(carrier, map);
                    }
                } else if is_sig_strength_obs_code!(observation) {
                    if let Some(vehicules) = ssi.get_mut(&carrier) {
                        if let Some(ssi) = vehicules.get_mut(vehicule) {
                            ssi.push(((e-e0) as f64, data.obs));
                        } else {
                            vehicules.insert(*vehicule, vec![((e-e0) as f64, data.obs)]);
                        }
                    } else {
                        let mut map: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
                        map.insert(*vehicule, vec![((e-e0) as f64, data.obs)]);
                        ssi.insert(carrier, map);
                    }
                }
            }
        }
    }
    if clock_offsets.len() > 0 { // got clock offsets
        let plot = ctx.plots.get("clock-offset.png")
            .expect("faulty plot context, missing clock offset plot");
        let mut chart = ctx.charts.get("CK")
            .expect("faulty plot context, missing clock offset chart")
            .clone()
            .restore(plot);
        chart
            .draw_series(LineSeries::new(
                clock_offsets.iter()
                    .map(|point| *point),
                &BLACK,
            ))
            .expect("failed to display receiver clock offsets")
            .label("Offset")
            .legend(|(x, y)| {
                //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
                PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
            });
        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(WHITE.filled())
            .draw()
            .expect("failed to draw clock offset labels");
    }
    for (carrier, vehicules) in phase {
        let symbol = symbols[carrier];
        for (sv, data) in vehicules {
            let color = ctx.colors.get(&sv.to_string())
                .expect(&format!("no colors to identify \"{}\"", sv));
            let plot = ctx.plots.get("phase.png")
                .expect("missing phase data plot");
            let mut chart = ctx.charts
                .get("PH")
                .expect("missing phase data chart")
                .clone()
                .restore(plot);
            // draw line serie
            chart
                .draw_series(LineSeries::new(
                    data.iter()
                        .map(|point| *point),
                        color.stroke_width(3),
                ))
                .expect("failed to draw phase observations")
                .label(sv.to_string())
                .legend(|(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], &color)
                });
            // draw symbols that empasize carrier signal
            chart
                .draw_series(
                    data.iter()
                        .map(|point| {
                            if symbol == "x" {
                                Cross::new(*point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            } else if symbol == "o" {
                                Circle::new(*point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            } else {
                                TriangleMarker::new(*point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            }
                        }))
                        .expect("failed to draw phase observations");
            chart
                .configure_series_labels()
                .border_style(&BLACK)
                .background_style(WHITE.filled())
                .draw()
                .expect("failed to draw clock offset labels");
        }
    }
    for (carrier, vehicules) in pr {
        let symbol = symbols[carrier];
        for (sv, data) in vehicules {
            let color = ctx.colors.get(&sv.to_string())
                .expect(&format!("no colors to identify \"{}\"", sv));
            let plot = ctx.plots.get("pseudo-range.png")
                .expect("missing pseudo range plot");
            let mut chart = ctx.charts
                .get("PR")
                .expect("missing pseudo range data chart")
                .clone()
                .restore(plot);
            // draw line serie
            chart
                .draw_series(LineSeries::new(
                    data.iter()
                        .map(|point| *point),
                        color.stroke_width(3),
                ))
                .expect("failed to draw pseudo range observations")
                .label(sv.to_string())
                .legend(|(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], &color)
                });
            // draw symbols that empasize carrier signal
            chart
                .draw_series(
                    data.iter()
                        .map(|point| {
                            if symbol == "x" {
                                Cross::new(*point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            } else if symbol == "o" {
                                Circle::new(*point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            } else {
                                TriangleMarker::new(*point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            }
                        }))
                        .expect("failed to draw pseudo range observations");
            chart
                .configure_series_labels()
                .border_style(&BLACK)
                .background_style(WHITE.filled())
                .draw()
                .expect("failed to draw pseudo range labels");
        }
    }
    for (carrier, vehicules) in doppler {
        let symbol = symbols[carrier];
        for (sv, data) in vehicules {
            let color = ctx.colors.get(&sv.to_string())
                .expect(&format!("no colors to identify \"{}\"", sv));
            let plot = ctx.plots.get("doppler.png")
                .expect("missing doppler data plot");
            let mut chart = ctx.charts
                .get("DOP")
                .expect("missing doppler data chart")
                .clone()
                .restore(plot);
            // draw line serie
            chart
                .draw_series(LineSeries::new(
                    data.iter()
                        .map(|point| *point),
                        color.stroke_width(3),
                ))
                .expect("failed to draw doppler observations")
                .label(sv.to_string())
                .legend(|(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], &color)
                });
            // draw symbols that empasize carrier signal
            chart
                .draw_series(
                    data.iter()
                        .map(|point| {
                            if symbol == "x" {
                                Cross::new(*point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            } else if symbol == "o" {
                                Circle::new(*point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            } else {
                                TriangleMarker::new(*point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            }
                        }))
                        .expect("failed to draw doppler observations");
            chart
                .configure_series_labels()
                .border_style(&BLACK)
                .background_style(WHITE.filled())
                .draw()
                .expect("failed to draw doppler labels");
        }
    }
    for (carrier, vehicules) in ssi {
        let symbol = symbols[carrier];
        for (sv, data) in vehicules {
            let color = ctx.colors.get(&sv.to_string())
                .expect(&format!("no colors to identify \"{}\"", sv));
            let plot = ctx.plots.get("ssi.png")
                .expect("missing ssi data plot");
            let mut chart = ctx.charts
                .get("SSI")
                .expect("missing ssi data chart")
                .clone()
                .restore(plot);
            // draw line serie
            chart
                .draw_series(LineSeries::new(
                    data.iter()
                        .map(|(x, y)| (*x, *y)),
                        color.stroke_width(3),
                ))
                .expect("failed to draw ssi observations")
                .label(sv.to_string())
                .legend(|(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], &color)
                });
            // draw symbols that empasize carrier signal
            chart
                .draw_series(
                    data.iter()
                        .map(|point| {
                            if symbol == "x" {
                                Cross::new(*point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            } else if symbol == "o" {
                                Circle::new(*point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            } else {
                                TriangleMarker::new(*point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            }
                        }))
                        .expect("failed to draw ssi observations");
            chart
                .configure_series_labels()
                .border_style(&BLACK)
                .background_style(WHITE.filled())
                .draw()
                .expect("failed to draw ssi labels");
        }
    }
} 
