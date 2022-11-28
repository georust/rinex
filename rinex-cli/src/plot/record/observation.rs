//! Observation record plotting
use rinex::{
    *,
    prelude::*,
    navigation::*,
    observation::*,
};
use super::{
    Context, Plot2d, 
    build_plot,
    build_chart, 
    build_twoscale_chart, 
};
use plotters::{
    prelude::*,
    coord::Shift,
    chart::{
        ChartState,
        DualCoordChartState,
    },
};
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
 * Builds a plot context for Observation RINEX specificly.
 * Observation RINEX can be optionnaly enhanced with Navigation Data,
 * in this case we prefer 2Y axes plots
 */
pub fn build_context<'a> (dim: (u32, u32), record: &observation::Record, nav: &Option<Rinex>) -> Context<'a> {
    if let Some(nav) = nav {
        build_enhanced_context(dim, record, nav)
    } else {
        build_simple_context(dim, record)
    }
}

pub fn build_simple_context<'a> (dim: (u32, u32), record: &observation::Record) -> Context<'a> {
    let mut e0: f64 = 0.0;
    let mut t_axis: Vec<f64> = Vec::with_capacity(16384);
    let mut plots: HashMap<String, DrawingArea<BitMapBackend, Shift>> = HashMap::with_capacity(4);
    let mut y_ranges: HashMap<String, (f64, f64)> = HashMap::new();
    let mut charts: HashMap<String, ChartState<Plot2d>> = HashMap::new();

    //  => 1 plot per physics (ie., Observable)
    //     1 plot in case clock offsets were provided
    for (e_index, ((e, _flag), (clk_offset, vehicules))) in record.iter().enumerate() {
        if e_index == 0 {
            // store first epoch timestamp
            // to scale x_axis proplery (avoids fuzzy rendering)
            e0 = e.to_utc_seconds();
        }
        let t = e.to_utc_seconds() - e0;
        t_axis.push(t);

        // Build 1 plot in case Receiver Clock Offsets were provided
        // Associate 1 chart to each plot, for classical 2D x,y plot
        // Grab y range
        if let Some(clk_offset) = clk_offset {
            let title = "clock-offset.png";
            plots.insert(title.to_string(), build_plot(title, dim));
            if let Some((min, max)) = y_ranges.get_mut(title) {
                if clk_offset < min {
                    *min = *clk_offset;
                }
                if clk_offset > max {
                    *max = *clk_offset;
                }
            } else {
                y_ranges.insert("Clock Offset".to_string(), (*clk_offset, *clk_offset));
            }
        }

        // Build 1 plot per type of observation
        // Associate 1 chart to each plot, for classical
        //
        // Color space: one color per vehicule
        //    identified by PRN#
        for (_, observations) in vehicules {
            for (observation, data) in observations {
                if is_phase_carrier_obs_code!(observation) {
                    let file = "phase.png";
                    if plots.get(file).is_none() {
                        let plot = build_plot(file, dim);
                        plots.insert(file.to_string(), plot);
                    }
                    let y = data.obs;
                    if let Some((min, max)) = y_ranges.get_mut("Phase") {
                        if y < *min {
                            *min = y;
                        }
                        if y > *max {
                            *max = y;
                        }
                    } else {
                        y_ranges.insert("Phase".to_string(), (y, y));
                    }
                } else if is_doppler_obs_code!(observation) {
                    let file = "doppler.png";
                    if plots.get(file).is_none() {
                        let plot = build_plot(file, dim);
                        plots.insert(file.to_string(), plot);
                    }
                    if let Some((min, max)) = y_ranges.get_mut("Doppler") {
                        if data.obs < *min {
                            *min = data.obs;
                        }
                        if data.obs > *max {
                            *max = data.obs;
                        }
                    } else {
                        y_ranges.insert("Doppler".to_string(), (data.obs, data.obs));
                    }
                } else if is_pseudo_range_obs_code!(observation) {
                    let file = "pseudo-range.png";
                    if plots.get(file).is_none() {
                        let plot = build_plot(file, dim);
                        plots.insert(file.to_string(), plot);
                    }
                    if let Some((min, max)) = y_ranges.get_mut("Pseudo Range") {
                        if data.obs < *min {
                            *min = data.obs;
                        }
                        if data.obs > *max {
                            *max = data.obs;
                        }
                    } else {
                        y_ranges.insert("Pseudo Range".to_string(), (data.obs, data.obs));
                    }
                } else if is_sig_strength_obs_code!(observation) {
                    let file = "ssi.png";
                    if plots.get(file).is_none() {
                        let plot = build_plot(file, dim);
                        plots.insert(file.to_string(), plot);
                    }
                    if let Some((min, max)) = y_ranges.get_mut("Signal Strength") {
                        if data.obs < *min {
                            *min = data.obs;
                        }
                        if data.obs > *max {
                            *max = data.obs;
                        }
                    } else {
                        y_ranges.insert("Signal Strength".to_string(), (data.obs, data.obs));
                    }
                }
            }
        }
    }
    // Add 1 chart onto each plot
    for (title, plot) in plots.iter() {
        let chart_id = match title.as_str() {
            "phase.png" => "Phase",
            "doppler.png" => "Doppler",
            "pseudo-range.png" => "Pseudo Range",
            "ssi.png" => "Signal Strength",
            "clock-offset.png" => "Clock Offset",
            _ => continue,
        };
        // scale this chart nicely
        let range = y_ranges.get(chart_id).unwrap();
        let chart = build_chart(chart_id, t_axis.clone(), *range, plot);
        charts.insert(chart_id.to_string(), chart);
    }
    Context {
        plots,
        charts,
        t_axis,
        dual_charts: HashMap::new(), // unused
    }
}

/*
 * in enhanced context, we take advantage of provided ephemeris
 * to plot Sv Elevation angles at the same time
 */
pub fn build_enhanced_context<'a> (dim: (u32, u32), record: &observation::Record, nav: &Rinex) -> Context<'a> {
    let mut e0: f64 = 0.0;
    let mut t_axis: Vec<f64> = Vec::with_capacity(16384);
    let mut plots: HashMap<String,
        DrawingArea<BitMapBackend, Shift>>
            = HashMap::with_capacity(4);
    
    let mut yr_ranges: HashMap<String, (f64,f64)> = HashMap::new();
    let mut charts: HashMap<String, ChartState<Plot2d>> = HashMap::new();
    
    let mut yl_range = (0.0_f64, 0.0_f64);
    let mut dual_charts: HashMap<String, DualCoordChartState<Plot2d, Plot2d>> = HashMap::new();

    let nav_rec = nav.record.as_nav()
        .expect("--nav should be Navigation Data");

    //  => 1 plot per physics (ie., Observable)
    //     1 plot in case clock offsets were provided
    for (e_index, ((e, _flag), (clk_offset, vehicules))) in record.iter().enumerate() {
        if e_index == 0 {
            // store first epoch timestamp
            // to scale x_axis proplery (avoids fuzzy rendering)
            e0 = e.to_utc_seconds();
        }
        let t = e.to_utc_seconds() - e0;
        t_axis.push(t);

        // Build 1 plot in case Receiver Clock Offsets were provided 
        // Associate 1 chart to each plot, for classical 2D x,y plot 
        // Grab y range
        if let Some(clk_offset) = clk_offset {
            let title = "clock-offset.png";
            plots.insert(
                title.to_string(),
                build_plot(title, dim));
            if let Some((min,max)) = yr_ranges.get_mut(title) {
                if clk_offset < min {
                    *min = *clk_offset;
                }
                if clk_offset > max {
                    *max = *clk_offset;
                }

            } else {
                yr_ranges.insert("Clock Offset".to_string(),
                    (*clk_offset,*clk_offset));
            }
        }
        
        let nav_epoch = nav_rec.get(e); 

        // Build 1 plot per type of observation
        // Associate 1 chart to each plot, for classical 
        //
        // Color space: one color per vehicule
        //    identified by PRN#
        for (sv, observations) in vehicules {
            /*
             * try to grab related ephemeris data
             */
            if let Some(nav_classes) = nav_epoch {
                for (class, frames) in nav_classes {
                    if *class == FrameClass::Ephemeris {
                        for fr in frames {
                            let (_, nav_sv, eph) = fr.as_eph()
                                .unwrap();
                            if nav_sv == sv {
                                if let Some(e) = eph.elevation_angle() {
                                    if e < yl_range.0 {
                                        yl_range.0 = e;
                                    }
                                    if e > yl_range.1 {
                                        yl_range.1 = e;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            for (observation, data) in observations {
                if is_phase_carrier_obs_code!(observation) {
                    let file = "phase.png";
                    if plots.get(file).is_none() {
                        let plot = build_plot(file, dim);
                        plots.insert(file.to_string(), plot);
                    }
                    let y = data.obs;
                    if let Some((min,max)) = yr_ranges.get_mut("Phase") {
                        if y < *min {
                            *min = y;
                        }
                        if y > *max {
                            *max = y;
                        }
                    } else {
                        yr_ranges.insert("Phase".to_string(), (y,y));
                    }
                } else if is_doppler_obs_code!(observation) {
                    let file = "doppler.png";
                    if plots.get(file).is_none() {
                        let plot = build_plot(file, dim);
                        plots.insert(file.to_string(), plot);
                    }
                    if let Some((min,max)) = yr_ranges.get_mut("Doppler") {
                        if data.obs < *min {
                            *min = data.obs;
                        }
                        if data.obs > *max {
                            *max = data.obs;
                        }
                    } else {
                        yr_ranges.insert("Doppler".to_string(),
                            (data.obs,data.obs));
                    }
                } else if is_pseudo_range_obs_code!(observation) {
                    let file = "pseudo-range.png";
                    if plots.get(file).is_none() {
                        let plot = build_plot(file, dim);
                        plots.insert(file.to_string(), plot);
                    }
                    if let Some((min,max)) = yr_ranges.get_mut("Pseudo Range") {
                        if data.obs < *min {
                            *min = data.obs;
                        }
                        if data.obs > *max {
                            *max = data.obs;
                        }
                    } else {
                        yr_ranges.insert("Pseudo Range".to_string(),
                            (data.obs,data.obs));
                    }
                } else if is_sig_strength_obs_code!(observation) {
                    let file = "ssi.png";
                    if plots.get(file).is_none() {
                        let plot = build_plot(file, dim);
                        plots.insert(file.to_string(), plot);
                    }
                    if let Some((min,max)) = yr_ranges.get_mut("Signal Strength") {
                        if data.obs < *min {
                            *min = data.obs;
                        }
                        if data.obs > *max {
                            *max = data.obs;
                        }
                    } else {
                        yr_ranges.insert("Signal Strength".to_string(),
                            (data.obs,data.obs));
                    }
                }
            }
        }
    }
    // Add 1 chart onto each plot
    for (title, plot) in plots.iter() {
        let chart_id = match title.as_str() {
            "phase.png" => "Phase",
            "doppler.png" => "Doppler",
            "pseudo-range.png" => "Pseudo Range",
            "ssi.png" => "Signal Strength",
            "clock-offset.png" => "Clock Offset",
            _ => continue,
        };
        // scale this chart nicely
        let range = yr_ranges.get(chart_id)
            .unwrap();
        let chart = build_twoscale_chart(chart_id, t_axis.clone(), (*range, yl_range), plot);
        dual_charts.insert(chart_id.to_string(), chart);
    }
    Context {
        plots,
        charts,
        t_axis,
        dual_charts,
    }
}

pub fn plot(ctx: &mut Context, record: &observation::Record, nav_ctx: &Option<Rinex>) {
    if let Some(nav) = nav_ctx {
        enhanced_plot(ctx, record, nav);
    } else {
        simple_plot(ctx, record);
    }
}

pub fn simple_plot(ctx: &mut Context, record: &observation::Record) {
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

    for (e_index, ((epoch, _flag), (clock_offset, vehicules))) in record.iter().enumerate() {
        if e_index == 0 {
            e0 = epoch.to_utc_seconds();
        }

        let e = epoch.to_utc_seconds();
        let x = e - e0;
        if let Some(value) = clock_offset {
            clk_offset.push((x, *value));
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
            .charts
            .get(&physics)
            .expect(&format!("faulty context: missing \"{}\" chart", physics))
            .clone()
            .restore(plot);
        /*
         * plot for this kind of data
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
                /* chart.draw_series(LineSeries::new(
                data.iter()
                    .map(|(_,x,y)| (*x,*y)),
                    color.clone()))
                .expect(&format!("failed to draw {} data", physics)); */
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

pub fn enhanced_plot(ctx: &mut Context, record: &observation::Record, nav: &Rinex) {
    let mut e0: f64 = 0.0;
    let cmap = colorous::TURBO; // to differentiate vehicules (PRN#)
    let symbols = vec!["x", "t", "o"]; // to differentiate carrier signals
    // sorted by Physics, By Carrier number, By vehicule, (x,y)
    let mut clk_offset: Vec<(f64,f64)> = Vec::new();
    // dataset
    //  per physics, per carrier signal (symbol)
    //      per vehicule (color map)
    //      x: sampling timestamp, 
    //      y: observation (raw), 
    //      bool: loss of lock - CS emphasis
    //      optionnal(f64): Sv elevation angle, if NAV is provided
    let mut dataset: HashMap<String, HashMap<u8, HashMap<Sv, Vec<(bool,f64,f64)>>>> = HashMap::new();

    let mut elev_angles: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
    let nav_rec = nav.record.as_nav()
        .expect("`--nav` should be navigation data");

    for (e_index, ((epoch, _flag), (clock_offset, vehicules))) in record.iter().enumerate() {
        if e_index == 0 {
            e0 = epoch.to_utc_seconds();
        }
        
        let e = epoch.to_utc_seconds();
        let x = e-e0;
        if let Some(value) = clock_offset {
            clk_offset.push((x, *value));
        }

        // retrieve related epoch in Nav
        let nav_classes = nav_rec.get(&epoch);
        if let Some(classes) = nav_classes {
            println!("{}: {}", epoch, classes.len());
        } else{
            println!("{}: NONE", epoch);
        }
        
        for (sv, observations) in vehicules {
            
            // retrieve related elevation angle
            if let Some(classes) = nav_classes {
                for (class, frames) in classes {
                    if *class == FrameClass::Ephemeris {
                        for fr in frames {
                            let (_, nav_sv, eph) = fr.as_eph()
                                .unwrap();
                            if nav_sv == sv {
                                if let Some(e) = eph.elevation_angle() {
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
                let c_code = u8::from_str_radix(c_code, 10)
                    .expect("failed to parse carrier code");
                
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
                        let mut map: HashMap<Sv, Vec<(bool,f64,f64)>> = HashMap::new();
                        map.insert(*sv, vec![(cycle_slip, x, y)]);
                        data.insert(c_code, map);
                    }
                } else {
                    let mut map: HashMap<Sv, Vec<(bool,f64,f64)>> = HashMap::new();
                    map.insert(*sv, vec![(cycle_slip, x, y)]);
                    let mut mmap: HashMap<u8, HashMap<Sv, Vec<(bool,f64,f64)>>> = HashMap::new();
                    mmap.insert(c_code, map);
                    dataset.insert(physics.to_string(), mmap);
                }
            }
        }
    }

    println!("Elev angles {:#?}", elev_angles);

    if let Some(plot) = ctx.plots.get("clock-offset.png") {
        let mut chart = ctx.charts.get("Clock Offset")
            .expect("faulty plot context: no chart defined for clock offsets")
            .clone()
            .restore(plot);
        chart.draw_series(
            clk_offset.iter()
                .map(|point| {
                    TriangleMarker::new(*point, 4,
                        Into::<ShapeStyle>::into(&BLACK).filled())
                    .into_dyn()
                }))
            .expect("failed to plot receiver clock offsets");
        chart
            .draw_series(LineSeries::new(
                clk_offset.iter()
                    .map(|point| *point),
                &BLACK,
            ))
            .expect("failed to plot receiver clock offsets")
            .label("Offset [s]")
            .legend(move |point| {
                TriangleMarker::new(point, 4,
                    Into::<ShapeStyle>::into(&BLACK).filled())
                    .into_dyn()
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
        let plot = ctx.plots.get(plot_title)
            .expect(&format!("faulty context: missing plot for \"{}\" data", physics)); 
        // retrieve associated chart
        let mut chart = ctx.dual_charts
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
                let color = RGBColor { 0: color.r, 1: color.g, 2: color.b };
                // plot
                chart.draw_series(
                    data.iter()
                        .map(|(cycle_slip,x,y)| {
                            if *cycle_slip {
                                match symbol {
                                    "x" => {
                                        Cross::new((*x,*y), 5,
                                            Into::<ShapeStyle>::into(&BLACK).filled().stroke_width(2))
                                            .into_dyn()
                                    },
                                    "o" => {
                                        Circle::new((*x,*y), 5,
                                            Into::<ShapeStyle>::into(&BLACK).filled().stroke_width(2))
                                            .into_dyn()
                                    },
                                    _ => {
                                        TriangleMarker::new((*x,*y), 5,
                                            Into::<ShapeStyle>::into(&BLACK).filled().stroke_width(2))
                                            .into_dyn()
                                    },
                                }
                            } else {
                                match symbol {
                                    "x" => {
                                        Cross::new((*x,*y), 4,
                                            Into::<ShapeStyle>::into(&color).filled())
                                            .into_dyn()
                                    },
                                    "o" => {
                                        Circle::new((*x,*y), 4,
                                            Into::<ShapeStyle>::into(&color).filled())
                                            .into_dyn()
                                    },
                                    _ => {
                                        TriangleMarker::new((*x,*y), 4,
                                            Into::<ShapeStyle>::into(&color).filled())
                                            .into_dyn()
                                    },
                                }
                            }
                        }))
                        .expect(&format!("failed to draw {} observations", physics))
                        .label(format!("{}(L{})", sv, carrier))
                        .legend(move |point| {
                            match symbol {
                                "x" => {
                                    Cross::new(point, 4,
                                        Into::<ShapeStyle>::into(&color).filled())
                                        .into_dyn()
                                },
                                "o" => {
                                    Circle::new(point, 4,
                                        Into::<ShapeStyle>::into(&color).filled())
                                        .into_dyn()
                                },
                                _ => {
                                    TriangleMarker::new(point, 4,
                                        Into::<ShapeStyle>::into(&color).filled())
                                        .into_dyn()
                                },
                            }
                        });

                        /*
                         * possible plot enhancement
                         */
                        if let Some(data) = elev_angles.get(&sv) {
                            // --> enhance this plot
                            chart.draw_secondary_series(
                                data.iter()
                                    .map(|(x, y)| {
                                        Circle::new((*x, *y), 5,
                                            Into::<ShapeStyle>::into(&RED).filled())
                                            .into_dyn()
                                }))
                                .expect(&format!("failed to enhance {} plot with elevation angle", physics))
                                .label(format!("{}", sv))
                                .legend(move |(x,y)| {
                                    Circle::new((x+10, y),
                                        Into::<ShapeStyle>::into(&RED).filled())
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
