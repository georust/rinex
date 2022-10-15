use rinex::{*,
    navigation::*,
    meteo::Observable,
};
use plotters::{
    prelude::*,
    coord::Shift,
    chart::ChartState,
    coord::types::RangedCoordf64,
};
use std::ops::Range;
use itertools::Itertools;
use std::collections::HashMap;

mod meteo;
mod navigation;
mod observation;

pub type Plot2d = Cartesian2d<RangedCoordf64, RangedCoordf64>;
 
pub struct Context<'a> {
    /// Plots are "Drawing areas" that we can either
    /// draw basic structures on, or stack Charts
    /// and 3D widgets onto.
    /// Plots are sorted by their titles which should always
    /// be a meaningful value
    pub plots: HashMap<String, DrawingArea<BitMapBackend<'a>, Shift>>,
    /// Drawing charts,
    /// is where actual plotting happens.
    /// We only work with f64 data
    pub charts: HashMap<String, ChartState<Plot2d>>,
    /// All plots share same time axis
    pub t_axis: Vec<f64>, 
    /// Colors used when plotting
    pub colors: HashMap<String, RGBAColor>,
}

impl Default for Context<'_> {
    fn default() -> Self {
        Self {
            t_axis: Vec::new(),
            colors: HashMap::new(),
            charts: HashMap::new(),
            plots: HashMap::new(),
        }
    }
}

impl<'a> Context<'a> {
    /// Builds a new plotting context
    ///  Iterates the RINEX context once (for overall performance considerations).
    ///  Prepares Plot and Charts depending on given RINEX context.
    ///  Currently all `Epoch` sorted RINEX have a time axis
    ///  in second, that eventually should be improved to exhibit
    ///  the real Date object. It seems possible to plot a Date<Local>
    ///  with the libs we're using.
    ///
    ///  Dim: (u32, u32) plot x_width and y_height
    pub fn new(dim: (u32,u32), rnx: &Rinex) -> Self {
        // holds time axis
        //   for Epoch Iterated RINEX files
        let mut e0: i64 = 0;
        let mut t_axis: Vec<f64> = Vec::with_capacity(16384);
        
        // Y axis range, for nicer rending 
        let mut y_range: HashMap<String, f64> = HashMap::with_capacity(4);
        
        // Plots / drawing areas
        // sorted by title
        let mut plots: HashMap<String,
            DrawingArea<BitMapBackend, Shift>>
                = HashMap::with_capacity(4);

        let mut y_ranges: HashMap<String, (f64,f64)> = HashMap::new();
        
        // Colors, one for each curve to be plotted,
        // identified by meaningful information
        let mut colors: HashMap<String, RGBAColor> = HashMap::with_capacity(32);

        let mut charts: HashMap<String, ChartState<Plot2d>> = HashMap::new();

        // build RINEX dependent context
        if let Some(record) = rnx.record.as_obs() {
            // Observation RINEX context
            //  => 1 plot per physics (ie., Observable)
            //     1 plot in case clock offsets were provided
            //  TODO
            //      emphasize LLI and SSI somehow ?
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
                        Self::build_plot(title, dim));
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
                                let plot = Self::build_plot(file, dim);
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
                                let plot = Self::build_plot(file, dim);
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
                                let plot = Self::build_plot(file, dim);
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
                                let plot = Self::build_plot(file, dim);
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
                let chart = Self::build_chart(chart_id, t_axis.clone(), *range, plot);
                charts.insert(chart_id.to_string(), chart);
            }
        
        } else if let Some(record) = rnx.record.as_nav() {
            for (index, (e, classes)) in record.iter().enumerate() {
                if index == 0 {
                    // store first epoch timestamp
                    // to scale x_axis proplery (avoids fuzzy rendering)
                    e0 = e.date.timestamp();
                }
                let t = e.date.timestamp() - e0;
                t_axis.push(t as f64);
                for (class, frames) in classes {
                    if *class == FrameClass::Ephemeris {
                        let file = "clock-bias.png";
                        if plots.get(file).is_none() {
                            let plot = Self::build_plot(file, dim);
                            plots.insert(file.to_string(), plot);
                        }
                        let file = "clock-drift.png";
                        if plots.get(file).is_none() {
                            let plot = Self::build_plot(file, dim);
                            plots.insert(file.to_string(), plot);
                        }
                        let file = "clock-driftr.png";
                        if plots.get(file).is_none() {
                            let plot = Self::build_plot(file, dim);
                            plots.insert(file.to_string(), plot);
                        }
                    }
                }
            }
            // Add one chart onto all plots
            for (id, plot) in plots.iter() {
                // scale this chart nicely
                let chart = Self::build_chart(id, t_axis.clone(), (-10.0E5, 10E5), plot);
                charts.insert(id.to_string(), chart);
            }
        } else if let Some(record) = rnx.record.as_meteo() {
            for (index, (e, observables)) in record.iter().enumerate() {
                if index == 0 {
                    // store first epoch timestamp
                    // to scale x_axis proplery (avoids fuzzy rendering)
                    e0 = e.date.timestamp();
                }
                let t = e.date.timestamp() - e0;
                t_axis.push(t as f64);
                for (observable, data) in observables {
                    if plots.get(&observable.to_string()).is_none() {
                        let title = match observable {
                            Observable::Pressure => "pressure.png",
                            Observable::Temperature => "temperature.png",
                            Observable::HumidityRate => "moisture.png",
                            Observable::ZenithWetDelay => "zenith-wet.png",
                            Observable::ZenithDryDelay => "zenith-dry.png",
                            Observable::ZenithTotalDelay => "zenith-total.png",
                            Observable::WindAzimuth => "wind-azim.png",
                            Observable::WindSpeed => "wind-speed.png",
                            Observable::RainIncrement => "rain-increment.png",
                            Observable::HailIndicator=> "hail.png",
                        };
                        let plot = Self::build_plot(title, dim);
                        plots.insert(observable.to_string(), plot);
                        y_ranges.insert(observable.to_string(), (*data, *data));
                    } else {
                        if let Some((min,max)) = y_ranges.get_mut(&observable.to_string()) {
                            if data < min {
                                *min = *data;
                            }
                            if data > max {
                                *max = *data;
                            }
                        } else {
                            y_ranges.insert(observable.to_string(), (*data, *data));
                        }
                    }
                }
            }
            // Add 1 chart onto each plot
            for (id, plot) in plots.iter() {
                // scale this chart nicely
                let range = y_ranges.get(id)
                    .unwrap();
                let chart = Self::build_chart(id, t_axis.clone(), *range, plot);
                charts.insert(id.to_string(), chart);
            }
        }
        Self {
            plots,
            charts,
            colors,
            t_axis,
        }
    }
    
    /// Build plot
    pub fn build_plot(file: &str, dim: (u32,u32)) -> DrawingArea<BitMapBackend, Shift> {
        let area = BitMapBackend::new(file, dim)
            .into_drawing_area();
        area.fill(&WHITE)
            .expect("failed to create background image");
        area
    }
    
    /// Build Charts
    pub fn build_chart(title: &str, 
        x_axis: Vec<f64>, 
        y_range: (f64,f64), 
        area: &DrawingArea<BitMapBackend, Shift>) 
    -> ChartState<Plot2d> {
        let x_axis = x_axis[0]..x_axis[x_axis.len()-1]; 
        let mut chart = ChartBuilder::on(area)
            .caption(title, ("sans-serif", 50).into_font())
            .margin(40)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(x_axis, 0.98*y_range.0..1.02*y_range.1) // nicer Y scale
            .unwrap();
        chart
            .configure_mesh()
            .x_desc("Timestamp [s]") //TODO not for special records
            .x_labels(30)
            .y_desc(title)
            .y_labels(30)
            .draw()
            .unwrap();
        chart
            .to_chart_state()
    }
}

pub fn plot_rinex(ctx: &mut Context, rnx: &Rinex) {
    if let Some(record) = rnx.record.as_obs() {
        observation::plot(ctx, record)
    } else if let Some(record) = rnx.record.as_nav() {
        navigation::plot(ctx, record)
    } else if let Some(record) = rnx.record.as_meteo() {
        meteo::plot(ctx, record)
    } else {
        panic!("this type of RINEX record cannot be plotted yet");
    }
}
