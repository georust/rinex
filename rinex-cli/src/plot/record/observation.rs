//! Observation record plotting
use rinex::*;
use rinex::observation::Record;
use super::{
    Context, Plot2d, 
    build_chart, build_plot,
};
use plotters::{
    prelude::*,
    coord::Shift,
    chart::ChartState,
};
use std::collections::HashMap;

/*
 * Builds a plot context for Observation RINEX specificly
 */
pub fn build_context<'a> (dim: (u32, u32), record: &Record) -> Context<'a> {
    let mut e0: i64 = 0;
    let mut t_axis: Vec<f64> = Vec::with_capacity(16384);
    let mut plots: HashMap<String,
        DrawingArea<BitMapBackend, Shift>>
            = HashMap::with_capacity(4);
    let mut y_ranges: HashMap<String, (f64,f64)> = HashMap::new();
    let mut colors: HashMap<String, RGBAColor> = HashMap::with_capacity(32);
    let mut charts: HashMap<String, ChartState<Plot2d>> = HashMap::new();

    //  => 1 plot per physics (ie., Observable)
    //     1 plot in case clock offsets were provided
    for (e_index, (e, (clk_offset, vehicules))) in record.iter().enumerate() {
        if e_index == 0 {
            // store first epoch timestamp
            // to scale x_axis proplery (avoids fuzzy rendering)
            e0 = e.date.timestamp();
        }
        let t = e.date.timestamp() - e0;
        t_axis.push(t as f64);

        // Build 1 plot in case Receiver Clock Offsets were provided 
        // Associate 1 chart to each plot, for classical 2D x,y plot 
        // Grab y range
        if let Some(clk_offset) = clk_offset {
            let title = "clock-offset.png";
            plots.insert(
                title.to_string(),
                build_plot(title, dim));
            if let Some((min,max)) = y_ranges.get_mut(title) {
                if clk_offset < min {
                    *min = *clk_offset;
                }
                if clk_offset > max {
                    *max = *clk_offset;
                }

            } else {
                y_ranges.insert("CK".to_string(),
                    (*clk_offset,*clk_offset));
            }
        }

        // Build 1 plot per type of observation
        // Associate 1 chart to each plot, for classical 
        //
        // Color space: one color per vehicule
        //    identified by PRN#
        for (v_index, (vehicule, observations)) in vehicules.iter().enumerate() {
            if colors.get(&vehicule.to_string()).is_none() {
                colors.insert(
                    vehicule.to_string(),
                    Palette99::pick(v_index) // RGB
                        .mix(0.99)); // => RGBA
            }
            for (observation, data) in observations {
                if is_phase_carrier_obs_code!(observation) {
                    let file = "phase.png";
                    if plots.get(file).is_none() {
                        let plot = build_plot(file, dim);
                        plots.insert(file.to_string(), plot);
                    }
                    if let Some((min,max)) = y_ranges.get_mut("PH") {
                        if data.obs < *min {
                            *min = data.obs;
                        }
                        if data.obs > *max {
                            *max = data.obs;
                        }
                    } else {
                        y_ranges.insert("PH".to_string(),
                            (data.obs,data.obs));
                    }
                } else if is_doppler_obs_code!(observation) {
                    let file = "doppler.png";
                    if plots.get(file).is_none() {
                        let plot = build_plot(file, dim);
                        plots.insert(file.to_string(), plot);
                    }
                    if let Some((min,max)) = y_ranges.get_mut("DOP") {
                        if data.obs < *min {
                            *min = data.obs;
                        }
                        if data.obs > *max {
                            *max = data.obs;
                        }
                    } else {
                        y_ranges.insert("DOP".to_string(),
                            (data.obs,data.obs));
                    }
                } else if is_pseudo_range_obs_code!(observation) {
                    let file = "pseudo-range.png";
                    if plots.get(file).is_none() {
                        let plot = build_plot(file, dim);
                        plots.insert(file.to_string(), plot);
                    }
                    if let Some((min,max)) = y_ranges.get_mut("PR") {
                        if data.obs < *min {
                            *min = data.obs;
                        }
                        if data.obs > *max {
                            *max = data.obs;
                        }
                    } else {
                        y_ranges.insert("PR".to_string(),
                            (data.obs,data.obs));
                    }
                } else if is_sig_strength_obs_code!(observation) {
                    let file = "ssi.png";
                    if plots.get(file).is_none() {
                        let plot = build_plot(file, dim);
                        plots.insert(file.to_string(), plot);
                    }
                    if let Some((min,max)) = y_ranges.get_mut("SSI") {
                        if data.obs < *min {
                            *min = data.obs;
                        }
                        if data.obs > *max {
                            *max = data.obs;
                        }
                    } else {
                        y_ranges.insert("SSI".to_string(),
                            (data.obs,data.obs));
                    }
                }
            }
        }
    }
    // Add 1 chart onto each plot
    for (title, plot) in plots.iter() {
        let chart_id = match title.as_str() {
            "phase.png" => "PH",
            "doppler.png" => "DOP",
            "pseudo-range.png" => "PR",
            "ssi.png" => "SSI",
            "clock-offset.png" => "CK",
            _ => continue,
        };
        // scale this chart nicely
        let range = y_ranges.get(chart_id)
            .unwrap();
        let chart = build_chart(chart_id, t_axis.clone(), *range, plot);
        charts.insert(chart_id.to_string(), chart);
    }
    Context {
        plots,
        charts,
        colors,
        t_axis,
    }
}

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
    /*
     * Plot phase observations
     */
    for (index, (_carrier, vehicules)) in phase.iter().enumerate() {
        for (sv_index, (sv, data)) in vehicules.iter().enumerate() {
            // one Symbol per Sv
            let symbol = symbols[sv_index % symbols.len()];
            // Sv color
            let color = ctx.colors.get(&sv.to_string())
                .expect(&format!("no colors to identify \"{}\"", sv));
            // retrieve plot
            let plot = ctx.plots.get("phase.png")
                .expect("missing phase data plot");
            let mut chart = ctx.charts
                .get("PH")
                .expect("missing phase data chart")
                .clone()
                .restore(plot);
            // plot
            if index == 0 {
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
                            .expect("failed to draw phase observations")
                            .label(sv.to_string())
                            .legend(|(x, y)| {
                                PathElement::new(vec![(x, y+10*sv_index as i32), (x + 20, y +10*sv_index as i32)], &color)
                            });
            } else {
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
            }
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
