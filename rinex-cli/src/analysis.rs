use rinex::*;
use super::plot::*;
use std::collections::HashMap;
use plotters::{
    prelude::*,
    coord::Shift,
    chart::ChartState,
    coord::types::RangedCoordf64,
};

/// Display vehicule per epoch
pub fn sv_epoch(rnx: &Rinex, nav: &mut Option<Rinex>, plot: bool, pretty: bool) {
    if let Some(nav) = nav {
        // special behavior
        if let Some(obs_rec) = rnx.record.as_obs() {
            // [1] retain ephemeris frames only
            //     in this context, we're only interested in Ephemeris frames
            if let Some(nav_rec) = nav.record.as_mut_nav() {
                if plot {
                    // plot will exhibit both Ephemeris and Observations
                    let mut p = build_plot("sv.png", (1024,768));
                    let min_date: i64 = 0;
                    let obs_epochs: Vec<Epoch> = obs_rec
                        .keys()
                        .map(|k| *k)
                        .collect();
                    let nav_epochs: Vec<Epoch> = nav_rec
                        .keys()
                        .map(|k| *k)
                        .collect();
                    // determine t_axis for nicer rendering
                    let mut max_date: f64 = 0.0;
                    if let Some(e0) = obs_epochs.get(0) {
                        if let Some(e) = obs_epochs.get(obs_epochs.len()-1) {
                            max_date = (e.date.timestamp() - e0.date.timestamp()) as f64
                        }
                    }
                    if let Some(e0) = nav_epochs.get(0) {
                        if let Some(e) = nav_epochs.get(nav_epochs.len()-1) {
                            let t = (e.date.timestamp() - e0.date.timestamp()) as f64;
                            if t > max_date {
                                max_date = t;
                            }
                        }
                    }
                    
                    // determine largest PRN # encountered
                    // and a color map per vehicules
                    let mut max_prn: u8 = 0;
                    let mut cmap: HashMap<Sv, RGBAColor> = HashMap::new(); 
                    for (_, (_, vehicules)) in obs_rec {
                        for (index, (sv, _)) in vehicules.iter().enumerate() {
                            if cmap.get(sv).is_none() {
                                cmap.insert(*sv, Palette99::pick(index)
                                    .mix(0.99));
                            }
                            if sv.prn > max_prn {
                                max_prn = sv.prn;
                            }
                        }
                    }
                    // drop non ephemeris frames at the same time
                    nav_rec.retain(|e, classes| {
                        classes.retain(|class, frames| {
                            if *class == navigation::FrameClass::Ephemeris {
                                for (index, frame) in frames.iter().enumerate() {
                                    let (_, sv, _) = frame.as_eph()
                                        .unwrap();
                                    if cmap.get(sv).is_none() {
                                        cmap.insert(*sv, Palette99::pick(index)
                                            .mix(0.99));
                                    }
                                    if sv.prn > max_prn {
                                        max_prn = sv.prn;
                                    }
                                }
                                true
                            } else {
                                false
                            }
                        });
                        classes.len() > 0 
                    });
                
                    let x_axis = 0.0..max_date;
                    let max_prn = max_prn as f64;
                    let y_axis = 0.0..max_prn;
                    let mut chart = ChartBuilder::on(&p)
                        .caption("Vehicules per Epoch", ("sans-serif", 50).into_font())
                        .margin(40)
                        .x_label_area_size(30)
                        .y_label_area_size(40)
                        .build_cartesian_2d(x_axis, y_axis)
                        .unwrap();
                    chart
                        .configure_mesh()
                        .x_desc("Timestamp [s]") //TODO not for special records
                        .x_labels(30)
                        .y_desc("Vehicule [Svnn]")
                        .y_labels(30)
                        .draw()
                        .expect("failed to draw mesh");
                    
                        // plot Ephemeris vehicules
                        //chart
                        //    .draw_series(nav_epochs.iter()
                        //        .map(|(

                } else { // NAV/OBS + no plot
                    nav_rec.retain(|e, classes| {
                        classes.retain(|class, _| *class == navigation::FrameClass::Ephemeris);
                        classes.len() > 0 
                    });
                
                    // stdout: shrink to shared epochs and print as is 
                    // shrink to Ephemeris and shared epochs
                    /*
                    nav_rec.retain(|e, vehicules| {
                        if let Some((_, obs_vehicules)) = obs_rec.get(e) {
                            vehicules.retain(|sv| {
                                let mut found = false;
                                for vehicule in obs_vehicules {
                                    found |= vehicule == sv;
                                }
                                found
                        });
                        vehicules.len() > 0
                        } else {
                            false
                        }
                    });
                
                    let content = match pretty {
                        true => serde_json::to_string_pretty(nav_data).unwrap(),
                        false => serde_json::to_string(nav_data).unwrap(),
                    };
                    println!("{}", content);
                    */
                }
            }
        }
    } else { // single file
        let data = rnx.space_vehicules_per_epoch();
        if plot {
            let mut p = build_plot("sv.png", (1024,768));
            let mut dates: (i64,i64) = (0, 0);
            let mut max_prn: u8 = 0;
            let mut cmap: HashMap<Sv, RGBAColor> = HashMap::new(); 
            for (e_index, (epoch, vehicules)) in data.iter().enumerate() {
                if e_index == 0 {
                    dates.0 = epoch.date.timestamp();
                } else {
                    dates.1 = epoch.date.timestamp() - dates.0;
                }
                for (sv_index, sv) in vehicules.iter().enumerate() {
                    if cmap.get(sv).is_none() {
                        cmap.insert(*sv, Palette99::pick(sv.prn.into())
                            .mix(0.99));
                    }
                    if sv.prn > max_prn {
                        max_prn = sv.prn;
                    }
                }
            }
            let t_axis = 0.0..(dates.1 as f64);
            let y_axis = 0.0..((max_prn +1) as f64);
            let mut chart = ChartBuilder::on(&p)
                .caption("Vehicules per Epoch", ("sans-serif", 50).into_font())
                .margin(40)
                .x_label_area_size(30)
                .y_label_area_size(40)
                .build_cartesian_2d(t_axis, y_axis)
                .unwrap();
            chart
                .configure_mesh()
                .x_desc("Timestamp [s]")
                .x_labels(30)
                .y_desc("Vehicule [Svnn]")
                .y_labels(30)
                .draw()
                .expect("failed to draw mesh");
            for (sv, color) in cmap.iter() {
                // grab this Svnn data
                let data: Vec<_> = data.iter()
                    .enumerate()
                    .filter_map(|(index, (epoch, vehicules))| {
                        if vehicules.contains(&sv) {
                            if index == 0 {
                                Some((0.0, sv.prn as f64))
                            } else {
                                Some(((epoch.date.timestamp() - dates.0) as f64, sv.prn as f64)) 
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                chart.draw_series(
                    data.iter()
                        .map(|point| {
                            Cross::new(*point, 4,
                                Into::<ShapeStyle>::into(&color).filled())
                                .into_dyn()
                        }))
                    .expect("failed to draw serie")
                    .label(sv.to_string())
                    .legend(|(x,y)| {
                        PathElement::new(vec![(x, y), (x + 20, y)], color.clone())
                    });
                chart
                    .configure_series_labels()
                    .border_style(&BLACK)
                    .background_style(WHITE.filled())
                    .draw()
                    .expect("failed to draw ssi labels");
            }
        } else {
            let content = match pretty {
                true => serde_json::to_string_pretty(&data).unwrap(),
                false => serde_json::to_string(&data).unwrap(),
            };
            println!("{}", content);
        }
    }
}
