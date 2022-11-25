use rinex::{
    prelude::*,
    navigation,
};
use crate::plot::*;
use std::collections::HashMap;
use plotters::prelude::*;

/// Display vehicules per epoch
pub fn analyze(rnx: &Rinex, nav: &mut Option<Rinex>, dims: (u32,u32)) {
    let mut cmap: HashMap<Sv, RGBAColor> = HashMap::new(); 
    let p = build_plot("sv.png", dims);

    if let Some(nav) = nav {
        // special behavior
        // Shared NAV/OBS epochs
        if let Some(obs_rec) = rnx.record.as_obs() {
            if let Some(nav_rec) = nav.record.as_mut_nav() {
                // plot will exhibit both Ephemeris and Observations
                let obs_epochs: Vec<Epoch> = obs_rec
                    .keys()
                    .map(|(k, _)| *k)
                    .collect();
               
                let obs_dates: (f64, f64) =
                    (obs_epochs[0].to_utc_seconds(),
                    obs_epochs[obs_epochs.len()-1].to_utc_seconds());
                    
                let nav_epochs: Vec<Epoch> = nav_rec
                    .keys()
                    .map(|k| *k)
                    .collect();
                
                let nav_dates: (f64, f64) =
                    (nav_epochs[0].to_utc_seconds(),
                    nav_epochs[nav_epochs.len()-1].to_utc_seconds());

                // determine t_axis for nicer rendering
                let mut max_date: f64 = 0.0;
                if let Some(e0) = obs_epochs.get(0) {
                    if let Some(e) = obs_epochs.get(obs_epochs.len()-1) {
                        max_date = e.to_utc_seconds() - e0.to_utc_seconds();
                    }
                }
                if let Some(e0) = nav_epochs.get(0) {
                    if let Some(e) = nav_epochs.get(nav_epochs.len()-1) {
                        let t = e.to_utc_seconds() - e0.to_utc_seconds();
                        if t > max_date {
                            max_date = t;
                        }
                    }
                }
                
                // determine largest PRN # encountered
                // and a color map per vehicules
                let mut max_prn: u8 = 0;
                for (_, (_, vehicules)) in obs_rec {
                    for (sv, _) in vehicules.iter() {
                        if cmap.get(sv).is_none() {
                            cmap.insert(*sv, Palette99::pick(sv.prn.into())
                                .mix(0.99));
                        }
                        if sv.prn > max_prn {
                            max_prn = sv.prn;
                        }
                    }
                }
                // drop non ephemeris frames at the same time
                nav_rec.retain(|_e, classes| {
                    classes.retain(|class, frames| {
                        if *class == navigation::FrameClass::Ephemeris {
                            for frame in frames {
                                let (_, sv, _) = frame.as_eph()
                                    .unwrap();
                                if cmap.get(sv).is_none() {
                                    cmap.insert(*sv, Palette99::pick(sv.prn.into())
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
                let y_axis = 0.0..((max_prn +1) as f64);
                let mut chart = ChartBuilder::on(&p)
                    .caption("Vehicules per Epoch", ("sans-serif", 50).into_font())
                    .margin(40)
                    .x_label_area_size(30)
                    .y_label_area_size(40)
                    .build_cartesian_2d(x_axis, y_axis)
                    .unwrap();
                chart
                    .configure_mesh()
                    .x_desc("Timestamp [s]")
                    .x_labels(30)
                    .y_desc("Vehicule [Svnn]")
                    .y_labels(30)
                    .draw()
                    .expect("failed to draw mesh");
                
                /*
                 * Plot Observation vehicules with cross symbol
                 */
                let data = rnx.space_vehicules_per_epoch();
                for (sv, color) in &cmap {
                    // grab this Svnn data
                    let data: Vec<_> = data.iter()
                        .enumerate()
                        .filter_map(|(index, (epoch, vehicules))| {
                            if vehicules.contains(&sv) {
                                if index == 0 {
                                    Some((0.0, sv.prn as f64))
                                } else {
                                    Some((epoch.to_utc_seconds() - obs_dates.0, sv.prn as f64)) 
                                }
                            }  else {
                                None
                            }
                        })
                        .collect();
                    chart.draw_series(
                        data.iter()
                            .map(|point| {
                                Cross::new(*point, 4, color.clone())
                            }))
                        .expect("failed to draw observation serie")
                        .label(sv.to_string().to_owned() + "(OBS)")
                        .legend(|(x, y)| {
                            Cross::new((x, y), 4, color.clone())
                        });
                }
                /*
                 * Plot Ephemeris vehicules with triangle symbol
                 */
                let data = nav.space_vehicules_per_epoch();
                for (sv, color) in &cmap {
                    // grab this Svnn data
                    let data: Vec<_> = data.iter()
                        .enumerate()
                        .filter_map(|(index, (epoch, vehicules))| {
                            if vehicules.contains(&sv) {
                                if index == 0 {
                                    Some((0.0, sv.prn as f64 +0.5))
                                } else {
                                    Some((epoch.to_utc_seconds() - nav_dates.0, sv.prn as f64 +0.5)) 
                                }
                            }  else {
                                None
                            }
                        })
                        .collect();
                    chart.draw_series(
                        data.iter()
                            .map(|point| {
                                TriangleMarker::new(*point, 4, color.clone())
                            }))
                        .expect("failed to draw navigation serie")
                        .label(sv.to_string().to_owned() + "(EPH)")
                        .legend(|(x, y)| {
                            TriangleMarker::new((x, y), 4, color.clone())
                        });
                }
                chart
                    .configure_series_labels()
                    .border_style(&BLACK)
                    .background_style(WHITE.filled())
                    .draw()
                    .expect("failed to draw ssi labels");
            }
        }
    } else { // single file
        let data = rnx.space_vehicules_per_epoch();
        
        // y axis offset, per constellation,
        //  to distinguish nicely
        let mut offset = 0.0_f64;
        let mut constell_offsets: HashMap<Constellation, f64> = HashMap::new();
        let constellations = rnx.list_constellations();
        let nb_constell = constellations.len() as f64;
        for cst in constellations {
            constell_offsets.insert(cst, offset);
            offset += 1.0 / nb_constell;
        }

        let mut dates: (f64,f64) = (0.0, 0.0);
        let mut max_prn: u8 = 0;
        for (e_index, (epoch, vehicules)) in data.iter().enumerate() {
            if e_index == 0 {
                dates.0 = epoch.to_utc_seconds()
            } else {
                dates.1 = epoch.to_utc_seconds() - dates.0;
            }
            for sv in vehicules.iter() {
                if cmap.get(sv).is_none() {
                    cmap.insert(*sv, Palette99::pick(sv.prn.into())
                        .mix(0.99));
                }
                if sv.prn > max_prn {
                    max_prn = sv.prn;
                }
            }
        }
        let t_axis = 0.0..dates.1;
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
                            Some(((epoch.to_utc_seconds() - dates.0) as f64, sv.prn as f64)) 
                        }
                    } else {
                        None
                    }
                })
                .collect();
            chart.draw_series(
                data.iter()
                    .map(|(x, y)| {
                        let p = (*x,
                            y + constell_offsets.get(&sv.constellation).unwrap());
                        match sv.constellation {
                            Constellation::GPS 
                            | Constellation::Geo 
                            | Constellation::SBAS(_) => {
                                Circle::new(p, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            },
                            Constellation::Glonass => {
                                Cross::new(p, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            },
                            _ => {
                                TriangleMarker::new(p, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()

                            },
                        }
                    }))
                .expect("failed to draw serie")
                .label(sv.to_string())
                .legend(|(x, y)| {
                    match sv.constellation {
                        Constellation::GPS 
                        | Constellation::Geo 
                        | Constellation::SBAS(_) => {
                            Circle::new((x,y), 4, 
                                Into::<ShapeStyle>::into(color.clone()).filled())
                                .into_dyn()
                        },
                        Constellation::Glonass => {
                            Cross::new((x,y), 4,
                                Into::<ShapeStyle>::into(color.clone()).filled())
                                .into_dyn()
                        },
                        _ => {
                            TriangleMarker::new((x,y), 4, 
                                Into::<ShapeStyle>::into(color.clone()).filled())
                                .into_dyn()
                        },
                    }
                });
            chart
                .configure_series_labels()
                .border_style(&BLACK)
                .background_style(WHITE.filled())
                .draw()
                .expect("failed to draw ssi labels");
        }
    }
}
