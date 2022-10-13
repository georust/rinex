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
//use plot::plot_record;
use extract::extract_data;
use retain::retain_filters;
use filter::apply_filters;
use resampling::record_resampling;

/*
                // One plot per physics
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
    let plot = cli.plot();
    let pretty = cli.pretty();

    let mut rnx = Rinex::from_file(cli.input_filepath())?;
    let mut ctx = plot::Context::default();

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
            ctx.set_time_axis(&rnx);
            ctx.set_y_range(&rnx);
            ctx.set_color_palette(&rnx);
            ctx.build_plot_areas((1024,768), &rnx);
            // Build a chart for every single identified area
            for (_, area) in ctx.areas.iter() {
                ctx.build_chart(area);
            }
            //ctx.build_plot(&rnx);
            //plot_record(&rnx, dim);
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
