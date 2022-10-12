//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Based on crate <https://github.com/gwbres/rinex>     
//! Homepage: <https://github.com/gwbres/rinex-cli>

use rinex::*; 
mod cli; // command line interface 
mod extract; // high level data
mod teqc; // `teqc` operations
mod plot; // plotting operations
mod retain; // record filtering 
mod filter; // record filtering
mod resampling; // record resampling

use cli::Cli;
use plot::plot_record;
use extract::extract_data;
use retain::retain_filters;
use filter::apply_filters;
use resampling::record_resampling;

/*
                // extract observations in form (e, observable, sv, data) 
                // for every single epoch, so we can plot easily
                // we scale epoch as unix timestamps, normalized to the 1st one encountered
                let data: Vec<_> = record
                    .iter()
                    .map(|(e, (clock_offset, vehicules))| {
                        vehicules.iter()
                            .map(|(sv, observables)| {
                                observables.iter()
                                    .map(|(observable, observation)| {
                                        ((e.date.timestamp() - e0.date.timestamp()) as f64, observable, sv.clone(), observation.obs)
                                    })
                            })
                            .flatten()
                    })
                    .flatten()
                    .collect();

                // Build one drawing base per physics
                // --> this will generate a file
                //  can't get this to be generic, due to some scope/lifetime issues...
                let ph_area = BitMapBackend::new(
                    "phase.png",
                    plot_dim)
                    .into_drawing_area();
                ph_area.fill(&WHITE)
                    .unwrap();
                
                let dop_area = BitMapBackend::new(
                    "doppler.png",
                    plot_dim)
                    .into_drawing_area();
                dop_area.fill(&WHITE)
                    .unwrap();
                
                let pr_area = BitMapBackend::new(
                    "pseudo-range.png",
                    plot_dim)
                    .into_drawing_area();
                pr_area.fill(&WHITE)
                    .unwrap();
                
                let ssi_area = BitMapBackend::new(
                    "ssi.png",
                    plot_dim)
                    .into_drawing_area();
                ssi_area.fill(&WHITE)
                    .unwrap();
                
                // Build one chart per physics
                //  chart is attached to the drawing base
                //  can't get this to be generic, due to some scope/lifetime issues...

                // build properly scaled Y axis
                let mut y_axis = default_axis.clone();
                if let Some((min,max)) = y_min_max.get("PH") {
                    y_axis = min*0.95..max*1.05;
                }
                let mut ph_chart = ChartBuilder::on(&ph_area)
                    .caption("Carrier Phase", ("sans-serif", 50).into_font())
                    .margin(40)
                    .x_label_area_size(30)
                    .y_label_area_size(40)
                    .build_cartesian_2d(
                        t_axis.clone(),
                        y_axis)
                    .unwrap();
                // Draw axes
                ph_chart
                    .configure_mesh()
                    .x_desc("Timestamp")
                    .x_labels(30)
                    //.y_label_formatter(&|y| format!("{:02}:{:02}", y.num_minutes(), y.num_seconds() % 60))
                    .y_desc("Phase")
                    .y_labels(30)
                    .draw()
                    .unwrap();

                let mut y_axis = default_axis.clone();
                if let Some((min, max)) = y_min_max.get("PR") {
                    y_axis = min*0.95..max*1.05;
                }
                let mut pr_chart = ChartBuilder::on(&pr_area)
                    .caption("Pseudo Range", ("sans-serif", 50).into_font())
                    .margin(40)
                    .x_label_area_size(30)
                    .y_label_area_size(40)
                    .build_cartesian_2d(
                        t_axis.clone(),
                        y_axis)
                    .unwrap();
                // Draw axes
                pr_chart
                    .configure_mesh()
                    .x_desc("Timestamp")
                    .x_labels(30)
                    //.y_label_formatter(&|y| format!("{:02}:{:02}", y.num_minutes(), y.num_seconds() % 60))
                    .y_desc("PR")
                    .y_labels(30)
                    .draw()
                    .unwrap();
                
                let mut y_axis = default_axis.clone(); 
                if let Some((min,max)) = y_min_max.get("DOP") {
                    y_axis = min*0.95..max*1.05;
                }
                let mut dop_chart = ChartBuilder::on(&dop_area)
                    .caption("Doppler", ("sans-serif", 50).into_font())
                    .margin(40)
                    .x_label_area_size(30)
                    .y_label_area_size(40)
                    .build_cartesian_2d(
                        t_axis.clone(),
                        y_axis)
                    .unwrap();
                // Draw axes
                dop_chart
                    .configure_mesh()
                    .x_desc("Timestamp")
                    .x_labels(30)
                    //.y_label_formatter(&|y| format!("{:02}:{:02}", y.num_minutes(), y.num_seconds() % 60))
                    .y_desc("Doppler")
                    .y_labels(30)
                    .draw()
                    .unwrap();
                
                let mut y_axis = default_axis.clone(); 
                if let Some((min,max)) = y_min_max.get("SSI") {
                    y_axis = min*0.95..max*1.05;
                }
                let mut ssi_chart = ChartBuilder::on(&ssi_area)
                    .caption("Signal Strength", ("sans-serif", 50).into_font())
                    .margin(40)
                    .x_label_area_size(30)
                    .y_label_area_size(40)
                    .build_cartesian_2d(
                        t_axis.clone(),
                        y_axis)
                    .unwrap();
                // Draw axes
                ssi_chart
                    .configure_mesh()
                    .x_desc("Timestamp")
                    .x_labels(30)
                    //.y_label_formatter(&|y| format!("{:02}:{:02}", y.num_minutes(), y.num_seconds() % 60))
                    .y_desc("Power")
                    .y_labels(30)
                    .draw()
                    .unwrap();

                // One plot per physics
                for observable in observables.iter() {
                    //  <o TODO
                    //     pick a symbol per carrier
                    if is_phase_carrier_obs_code!(observable) {
                        // Draw one serie per vehicule
                        for vehicule in vehicules.iter() {
                            // <o
                            //    pick a color per PRN#
                            let color = colors.get(vehicule)
                                .unwrap();
                            ph_chart.draw_series(LineSeries::new(
                                data.iter()
                                    .filter_map(|(t, obs, sv, data)| {
                                        if obs == observable {
                                            if sv == *vehicule {
                                                Some((*t, *data))
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    }), 
                                color.stroke_width(3),
                            )).unwrap()
                            .label(vehicule.to_string())
                            .legend(|(x, y)| {
                                let color = colors.get(vehicule).unwrap();
                                PathElement::new(vec![(x, y), (x + 20, y)], color)
                            });
                        }
                    } else if is_pseudo_range_obs_code!(observable) {
                        // Draw one serie per vehicule
                        for vehicule in vehicules.iter() {
                            // <o
                            //    pick a color per PRN#
                            let color = colors.get(vehicule)
                                .unwrap();
                            pr_chart.draw_series(LineSeries::new(
                                data.iter()
                                    .filter_map(|(t, obs, sv, data)| {
                                        if obs == observable {
                                            if sv == *vehicule {
                                                Some((*t, *data))
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    }), 
                                color.stroke_width(3),
                            )).unwrap()
                            .label(vehicule.to_string())
                            .legend(|(x, y)| {
                                let color = colors.get(vehicule).unwrap();
                                PathElement::new(vec![(x, y), (x + 20, y)], color)
                            });
                        }
                    } else if is_sig_strength_obs_code!(observable) {
                        // Draw one serie per vehicule
                        for vehicule in vehicules.iter() {
                            // <o
                            //    pick a color per PRN#
                            let color = colors.get(vehicule)
                                .unwrap();
                            ssi_chart.draw_series(LineSeries::new(
                                data.iter()
                                    .filter_map(|(t, obs, sv, data)| {
                                        if obs == observable {
                                            if sv == *vehicule {
                                                Some((*t, *data))
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    }), 
                                color.stroke_width(3),
                            )).unwrap()
                            .label(vehicule.to_string())
                            .legend(|(x, y)| {
                                let color = colors.get(vehicule).unwrap();
                                PathElement::new(vec![(x, y), (x + 20, y)], color)
                            });
                        }
                    } else if is_doppler_obs_code!(observable) {
                        // Draw one serie per vehicule
                        for vehicule in vehicules.iter() {
                            // <o
                            //    pick a color per PRN#
                            let color = colors.get(vehicule)
                                .unwrap();
                            dop_chart.draw_series(LineSeries::new(
                                data.iter()
                                    .filter_map(|(t, obs, sv, data)| {
                                        if obs == observable {
                                            if sv == *vehicule {
                                                Some((*t, *data))
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    }), 
                                color.stroke_width(3),
                            )).unwrap()
                            .label(vehicule.to_string())
                            .legend(|(x, y)| {
                                let color = colors.get(vehicule).unwrap();
                                PathElement::new(vec![(x, y), (x + 20, y)], color)
                            });
                        }
                    }
                }
                // Draw labels 
                ph_chart
                    .configure_series_labels()
                    .border_style(&BLACK)
                    .background_style(WHITE.filled())
                    .draw()
                    .unwrap();
                dop_chart
                    .configure_series_labels()
                    .border_style(&BLACK)
                    .background_style(WHITE.filled())
                    .draw()
                    .unwrap();
                pr_chart
                    .configure_series_labels()
                    .border_style(&BLACK)
                    .background_style(WHITE.filled())
                    .draw()
                    .unwrap();
                ssi_chart
                    .configure_series_labels()
                    .border_style(&BLACK)
                    .background_style(WHITE.filled())
                    .draw()
                    .unwrap();
            } // Observation Record
        } else {
            // terminal output
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rnx.record).unwrap())
            } else {
                println!("{}", serde_json::to_string(&rnx.record).unwrap())
            }
        }
    }
}
*/

pub fn main () -> Result<(), rinex::Error> {
    let cli = Cli::new();
    let mut rnx = Rinex::from_file(cli.input_filepath())?;
    let plot = cli.plot();
    let pretty = cli.pretty();

    if cli.resampling() { // resampling requested
        record_resampling(&mut rnx, cli.resampling_ops());
    }
    
    if cli.retain() { // retain data of interest
        retain_filters(&mut rnx, cli.retain_ops());
    }
    
    if cli.filter() { // apply desired filters
        apply_filters(&mut rnx, cli.filter_ops());
    }
    
    // grab data of interest
    if cli.extract() {
        extract_data(&rnx, cli.extraction_ops(), pretty);
    } else {
        // no data of interest
        // => extract record
        if plot {
            let dim = (1024, 768); //TODO: from CLI
            plot_record(&rnx, dim);
        } else {
            // print with desired option
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rnx.record).unwrap())
            } else {
                println!("{}", serde_json::to_string(&rnx.record).unwrap())
            }
        }
    }
    Ok(())
}// main

/*TODO manage multi file ?
    let filepaths : Option<Vec<&str>> = match matches.is_present("filepath") {
        true => {
            Some(matches.value_of("filepath")
                .unwrap()
                    .split(",")
                    .collect())
        },
        false => None,
    };


    // Header customization  
    let mut _custom_header: Option<Header> = None;
    if matches.is_present("custom-header") {
        let descriptor = matches.value_of("custom-header")
            .unwrap();
        match serde_json::from_str::<Header>(descriptor) {
            Ok(hd) => {
                _custom_header = Some(hd.clone());
            },
            Err(e) => {
                match std::fs::read_to_string(descriptor) {
                    Ok(content) => {
                        match serde_json::from_str::<Header>(&content) {
                            Ok(hd) => {
                                _custom_header = Some(hd.clone())
                            },
                            Err(ee) => panic!("failed to interprate header: {:?}", ee),
                        }
                    },
                    Err(_) => {
                        panic!("failed to interprate header: {:?}", e)
                    }
                }
            },
        }
    }

    let filepaths = filepaths
        .unwrap(); // input files are mandatory
    // work queue, contains all parsed RINEX
    let mut queue: Vec<Rinex> = Vec::new();

    /*let pretty = matches.is_present("pretty");

    // `ddiff` special ops,
    // is processed at very last, because it will eventuelly drop
    // all non Observation RINEX.
    // This requires 2 OBS and 1 NAV files
    if matches.is_present("ddiff") {
        let mut nav : Option<Rinex> = None;
        // tries to identify a NAV file in provided list 
        // this stupidly grabs the first one encountered
        for i in 0..queue.len() {
            if queue[i].is_navigation_rinex() {
                nav = Some(queue[i].clone());
            }
        }
        // 
        if let Some(nav) = nav { // got something
            // drop all other RNX
            queue.retain(|q| q.is_observation_rinex());
            // --> apply `ddiff` related to this NAV
            //     on each remaining --file a,b duplets
            for i in 0..queue.len() /2 {
                let q_2p = &queue[i*2];
                let q_2p1 = &queue[i*2+1];
                let ddiff = q_2p.double_diff(q_2p1, &nav);
                if ddiff.is_ok() {
                    // currently just prints the record
                    // but we'll unlock plotting in next releases
                    let rnx = ddiff.unwrap();
                    let rec = rnx.record.as_obs().unwrap();
                    if pretty {
                        println!("{}", serde_json::to_string_pretty(&rec).unwrap())
                    } else {
                        println!("{}", serde_json::to_string(&rec).unwrap())
                    }
                } else {
                    panic!("--ddiff panic'ed with {:?}", ddiff);
                }
            }
        } else {
            panic!("--ddiff requires NAV ephemeris to be provided!");
        }
    }*/
*/    
