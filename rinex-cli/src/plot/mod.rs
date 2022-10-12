use rinex::*;
use plotters::{
    prelude::*,
    coord::Shift,
    coord::types::RangedCoordf64,
};
use std::ops::Range;

//mod meteo;
//mod navigation;
mod observation;
use itertools::Itertools;

use std::rc::Rc;
use std::collections::HashMap;

pub type Chart<'a> = ChartContext<'a, BitMapBackend<'a>,
    Cartesian2d<RangedCoordf64, RangedCoordf64>>;
    
pub struct Context<'a> {
    /// Drawing areas,
    /// will eventually generate a .PNG or .SVG
    /// file, depending on backend being used
    pub areas: HashMap<String, DrawingArea<BitMapBackend<'a>, Shift>>,
    /// Drawing charts,
    /// is where actual plotting happens.
    /// We only work with f64 data
    pub charts: HashMap<String, Rc<Chart<'a>>>,
    /// Colors used when plotting
    pub colors: HashMap<String, RGBAColor>,
    /// All plots share same time axis
    pub t_axis: Range<f64>,
    /// Structure to scale datasets nicely.
    /// Holds (min,max) values per identified datasets
    pub y_ranges: HashMap<String, (f64,f64)>,
    /// List of vehicules contained in record,
    /// Helps identify datasets
    pub vehicules: Vec<Sv>,
}

impl Default for Context<'_> {
    fn default() -> Self {
        Self {
            areas: HashMap::new(),
            charts: HashMap::new(),
            colors: HashMap::new(),
            t_axis: (0.0..10.0),
            vehicules: Vec::new(),
            y_ranges: HashMap::new(),
        }
    }
}

impl Context<'_> {
    fn build_time_axis(rnx: &Rinex) -> Range<f64> {
        let e0 = rnx.first_epoch()
            .unwrap();
        if let Some(record) = rnx.record.as_obs() {
            let timestamps: Vec<_> = record
                .iter()
                .map(|(e, _)| {
                    (e.date.timestamp() - e0.date.timestamp()) as f64
                })
                .collect();
            timestamps[0]..timestamps[timestamps.len()-1]
        } else {
            0.0..10.0
        }
    }
    /// Builds a new RINEX dependent
    /// plotting context
    pub fn new(rnx: &Rinex, dim:(u32,u32)) -> Self {
        let t_axis = Self::build_time_axis(&rnx); 
        let mut colors: HashMap<String, RGBAColor> 
            = HashMap::new();
        let mut areas: HashMap<String, DrawingArea<BitMapBackend, Shift>> 
            = HashMap::new();
        let mut charts: HashMap<String,
            ChartContext<BitMapBackend,
                Cartesian2d<RangedCoordf64, RangedCoordf64>>>
                    = HashMap::new();
        if let Some(record) = rnx.record.as_obs() {
            // Observation RINEX context
            //  1 area/1 plot per physics, ie. Observables
            let vehicules: Vec<Sv> = record
                .iter()
                .map(|(_, (_, vehicules))| {
                    vehicules.iter() 
                        .map(|(sv, _)| *sv)
                })
                .flatten()
                .unique()
                .collect();
            // smart color generation
            //  indexed on PRN#
            for (index, sv) in vehicules.iter().enumerate() {
                colors.insert(sv.to_string(),
                    Palette99::pick(index) // RGB
                        .mix(0.99)); // =>RGBA
            }
            // Build Y axis range
            let mut y_ranges: HashMap<String, (f64,f64)> =
                HashMap::with_capacity(4); // 4 known physics
            for (_, (_, vehicules)) in record.iter() {
                for (_, observables) in vehicules.iter() {
                    for (code, data) in observables.iter() {
                        if is_pseudo_range_obs_code!(code) {
                            if let Some((min,max)) = y_ranges.get_mut("PR") {
                                if *min > data.obs {
                                    *min = data.obs;
                                }
                                if *max < data.obs {
                                    *max = data.obs;
                                }
                            } else {
                                y_ranges.insert(
                                    "PR".to_string(),
                                    (data.obs,data.obs));
                            }
                        } else if is_phase_carrier_obs_code!(code) {
                            if let Some((min,max)) = y_ranges.get_mut("PH") {
                                if *min > data.obs {
                                    *min = data.obs;
                                }
                                if *max < data.obs {
                                    *max = data.obs;
                                }
                            } else {
                                y_ranges.insert(
                                    "PR".to_string(),
                                    (data.obs,data.obs));
                            }
                        } else if is_doppler_obs_code!(code) {
                            if let Some((min,max)) = y_ranges.get_mut("DOP") {
                                if *min > data.obs {
                                    *min = data.obs;
                                }
                                if *max < data.obs {
                                    *max = data.obs;
                                }
                            } else {
                                y_ranges.insert(
                                    "DOP".to_string(),
                                    (data.obs,data.obs));
                            }
                        } else {
                            if let Some((min,max)) = y_ranges.get_mut("SSI") {
                                if *min > data.obs {
                                    *min = data.obs;
                                }
                                if *max < data.obs {
                                    *max = data.obs;
                                }
                            } else {
                                y_ranges.insert(
                                    "DOP".to_string(),
                                    (data.obs,data.obs));
                            }
                        }
                    }
                }
            }
            // Build one drawing area and one chart per physics,
            // ie., observation kind
            for (identifier, (min,max)) in y_ranges.iter() {
                let area = BitMapBackend::new("TODO.png", dim)
                    .into_drawing_area();
                area.fill(&WHITE)
                    .unwrap();
                areas
                    .insert(identifier.to_string(), area.clone());
            }

            for (identifier, (min, max)) in y_ranges.iter() {
                if let Some(area) = areas.get_mut(identifier) {
                    // Build one chart 
                    let mut chart = ChartBuilder::on(area)
                    .caption(identifier, ("sans-serif", 50).into_font())
                    .margin(40)
                    .x_label_area_size(30)
                    .y_label_area_size(40)
                    .build_cartesian_2d(
                        t_axis.clone(),
                        0.95*min..1.05*max)
                    .unwrap();
                // Draw axes
                chart
                    .configure_mesh()
                    .x_desc("Timestamp [s]")
                    .x_labels(30)
                    .y_desc(identifier)
                    .y_labels(30)
                    .draw()
                    .unwrap();
                charts
                    .insert(identifier.to_string(), chart);
                }
            }
            Self {
                areas: HashMap::new(), //TODO conclude
                charts: HashMap::new(), //TODO conclude 
                colors,
                vehicules,
                t_axis,
                y_ranges,
            }
        /*} else if let Some(record) = rnx.record.as_meteo() {
            // Meteo RINEX context
            //  1 area/1 plot per physics, ie. Observables
            Self {
                colors,
                vehicules: Vec::new(), // unused
                t_axis: Self::build_time_axis(&rnx),    
                y_ranges,
            }
        } else if let Some(record) = rnx.record.as_nav() {
            // Navigation RINEX context (Ephemeris)
            //  ==> other frames not supported yet
            //  1 area/1 plot per physics, ie. Orbits fields
            //  1 plot for clock biases
            //  1 plot for clock drift
            //  1 plot for clock drift changes
            let vehicules: Vec<Sv> = record
                .iter()
                .map(|(_, (_, vehicules))| {
                    vehicules.iter() 
                        .map(|(sv, _)| sv)
                })
                .flatten()
                .unique()
                .collect();
            // smart color generation
            //  indexed on PRN#
            for (index, sv) in vehicules.iter().enumerate() {
                colors.insert(**sv,
                    Palette99::pick(index) // RGB
                        .mix(0.99)); // =>RGBA
            }
            Self {
                colors,
                vehicules,
                t_axis: Self::build_time_axis(&rnx),    
            }*/
        } else {
            Self::default()
        }
    }
}

pub fn plot_record(rnx: &Rinex, dim: (u32,u32)) {
    // create new RINEX dependent plotting context
    let mut ctx = Context::new(&rnx, dim);
    if let Some(record) = rnx.record.as_obs() {
        observation::plot(ctx, record)
    /*} else if let Some(record) = rnx.record.as_nav() {
        navigation::plot(record)
    } else if let Some(record) = rnx.record.as_meteo() {
        meteo::plot(record)*/
    } else {
        panic!("this type of RINEX record cannot be plotted yet");
    }
}
