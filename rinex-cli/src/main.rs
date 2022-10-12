//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Based on crate <https://github.com/gwbres/rinex>     
//! Homepage: <https://github.com/gwbres/rinex-cli>
use plotters::{
    prelude::*,
    //coord::Shift,
    //coord::types::RangedCoordf64,
};
use std::str::FromStr;
use std::collections::HashMap;
use itertools::Itertools;

use rinex::{*, 
    observation::Ssi, observation::LliFlags,
};

// command line interface
mod cli; 
use cli::Cli;

// high level extraction
mod extract;
use extract::extract_data;

// retain filters
mod retain;
use retain::retain_filters;

// record resampling
mod resampling;

mod parser; // user input parser
mod ascii_plot; // `teqc` tiny plot

/*
/// Resample given file as possibly requested
fn resample_single_file (rnx: &mut Rinex, matches: clap::ArgMatches) {
    if let Some(hms) = matches.value_of("decim-interval") { 
        if let Ok(interval) = parser::parse_duration(hms) {
            rnx
                .decimate_by_interval_mut(interval)
        }
    }
    if let Some(r) = matches.value_of("decim-ratio") {
        if let Ok(r) = u32::from_str_radix(r, 10) {
            rnx
                .decimate_by_ratio_mut(r)
        }
    }
}

/// Apply desired filters
fn apply_filters (rinex: &mut Rinex, matches: clap::ArgMatches) {
    let retain_epoch_ok = matches.is_present("retain-epoch-ok");
    let retain_epoch_nok = matches.is_present("retain-epoch-nok");
    let retain_constell : Option<Vec<Constellation>> = match matches.value_of("retain-constell") {
        Some(s) => {
            let constellations: Vec<&str> = s.split(",").collect();
            let mut c_filters : Vec<Constellation> = Vec::new();
            for c in constellations {
                if let Ok(constell) = Constellation::from_3_letter_code(c) {
                    c_filters.push(constell)
                } else if let Ok(constell) = Constellation::from_1_letter_code(c) {
                    c_filters.push(constell)
                }
            }
            Some(c_filters)
        },
        _ => None,
    };
    
    let retain_sv : Option<Vec<Sv>> = match matches.value_of("retain-sv") {
        Some(s) => {
            let sv: Vec<&str> = s.split(",").collect();
            let mut sv_filters : Vec<Sv> = Vec::new();
            for s in sv {
                let constell = Constellation::from_str(&s[0..1])
                    .unwrap();
                let prn = u8::from_str_radix(&s[1..], 10)
                    .unwrap();
                sv_filters.push(Sv::new(constell,prn))
            }
            Some(sv_filters)
        },
        _ => None,
    };

    let retain_obs : Option<Vec<&str>> = match matches.value_of("retain-obs") {
        Some(s) => Some(s.split(",").collect()),
        _ => None,
    };
    let retain_orb : Option<Vec<&str>> = match matches.value_of("retain-orb") {
        Some(s) => Some(s.split(",").collect()),
        _ => None,
    };
    let lli_and_mask : Option<u8> = match matches.value_of("lli-andmask") {
        Some(s) => Some(u8::from_str_radix(s,10).unwrap()),
        _ => None,
    };
    let ssi_filter : Option<Ssi> = match matches.value_of("ssi-filter") {
        Some(s) => Some(Ssi::from_str(s).unwrap()),
        _ => None,
    };

    let retain_nav_msg: Option<Vec<navigation::MsgType>> = match matches.value_of("retain-nav-msg") {
        Some(s) => {
            let mut filter: Vec<navigation::MsgType> = Vec::new();
            let descriptor: Vec<&str> = s.split(",").collect();
            for item in descriptor {
                if let Ok(msg) = navigation::MsgType::from_str(item) {
                    filter.push(msg)       
                }
            }
            if filter.len() == 0 {
                None
            } else {
                Some(filter.clone())
            }
        },
        _ => None,
    };

    let retain_legacy_nav = matches.is_present("retain-lnav");
    let retain_modern_nav = matches.is_present("retain-mnav");
    
    if retain_epoch_ok {
        rinex.retain_epoch_ok_mut()
    }
    if retain_epoch_nok {
        rinex.retain_epoch_nok_mut()
    }
    if let Some(ref filter) = retain_constell {
        rinex.retain_constellation_mut(filter.to_vec())
    }
    if let Some(ref filter) = retain_sv {
        rinex.retain_space_vehicule_mut(filter.to_vec())
    }
    if let Some(ref filter) = retain_obs {
        rinex.retain_observable_mut(filter.to_vec())
    }
    if let Some(ref filter) = retain_nav_msg {
        rinex.retain_navigation_message_mut(filter.to_vec())
    }
    if retain_legacy_nav {
        rinex.retain_legacy_navigation_mut();
    }
    if retain_modern_nav {
        rinex.retain_modern_navigation_mut();
    }
    if let Some(ref filter) = retain_orb {
        rinex.retain_ephemeris_orbits_mut(filter.to_vec())
    }
    if let Some(lli) = lli_and_mask {
        let mask = LliFlags::from_bits(lli)
            .unwrap();
        rinex.observation_lli_and_mask_mut(mask)
    }
    if let Some(ssi) = ssi_filter {
        rinex.minimum_sig_strength_filter_mut(ssi)
    }
}

/*TODO
can't get this to work due to a scope pb..
let mut areas: HashMap<String, //BitMapBackend> 
    DrawingArea<BitMapBackend, Shift>> 
        = HashMap::new();

let mut charts: HashMap<String, 
    ChartContext<BitMapBackend, 
        Cartesian2d<RangedCoordf64, RangedCoordf64>>>
            = HashMap::new();
*/
    if !at_least_one_op {
        // user did not request one of the high level ops
        // ==> either print or plot record data
        if plot { // visualization requested
            let record = &rnx.record;
            
            //TODO
            // could slightly improve here,
            // if given record is Sv sorted; by grabing list of vehicules
            // and pre allocating fixed size
            let mut colors: HashMap<Sv, RGBAColor> = HashMap::new();

            let default_axis = 0.0..10.0; // this is only here
                // to manage possible missing observables
                // versus the creation of properly scaled Y axis

            //TODO
            // from command line
            let plot_dim = (1024, 768);
            
            // Visualize all known RINEX records
            if let Some(record) = record.as_obs() {
                // form t axis
                // we use t.epochs.date as Unix timestamps
                //  normalized for 1st one encountered to get a nicer rendering
                let e0 = rnx.first_epoch().unwrap();
                let timestamps: Vec<_> = record.iter()
                    .map(|(epoch, _)| {
                        (epoch.date.timestamp() - e0.date.timestamp()) as f64
                    })
                    .collect();
                let t_axis = timestamps[0]..timestamps[timestamps.len()-1];
                
                // extra list of vehicules
                //  this will help identify datasets 
                let vehicules: Vec<_> = record
                    .iter()
                    .map(|(_, (_, vehicules))| {
                        vehicules.iter()
                            .map(|(sv, _)| sv)
                    })
                    .flatten()
                    .unique()
                    .collect();
                
                // smart color generation
                //  for PRN# identification
                for (index, sv) in vehicules.iter().enumerate() {
                    colors.insert(**sv, 
                        Palette99::pick(index) //RGB
                        .mix(0.99)); //RGB=RGBA
                }
                
                // extra list of encountered observables
                //  this will help identify datasets 
                let observables: Vec<_> = record
                    .iter()
                    .map(|(_, (_, vehicules))| {
                        vehicules.iter()
                            .map(|(_, observables)| {
                                observables.iter()
                                    .map(|(obscode, _)| obscode)
                            })
                            .flatten()
                    })
                    .flatten()
                    .unique()
                    .collect();

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

                // determine (min, max) per Observation Kind
                //   this is used to scale Y axis nicely
                let mut y_min_max: HashMap<String, (f64,f64)> = HashMap::with_capacity(4); // 4 physics known
                for (_, (_, vehicules)) in record.iter() {
                    for (_, observables) in vehicules.iter() {
                        for (code, data) in observables.iter() {
                            if is_pseudo_range_obs_code!(code) {
                                if let Some((min,max)) = y_min_max.get_mut("PR") {
                                    if *min > data.obs {
                                        *min = data.obs ;
                                    }
                                    if *max < data.obs {
                                        *max = data.obs ;
                                    }
                                } else {
                                    y_min_max.insert("PR".to_string(), (data.obs, data.obs));
                                }
                            } else if is_phase_carrier_obs_code!(code) {
                                if let Some((min,max)) = y_min_max.get_mut("PH") {
                                    if *min > data.obs {
                                        *min = data.obs ;
                                    }
                                    if *max < data.obs {
                                        *max = data.obs ;
                                    }
                                } else {
                                    y_min_max.insert("PH".to_string(), (data.obs, data.obs));
                                }
                            } else if is_doppler_obs_code!(code) {
                                if let Some((min,max)) = y_min_max.get_mut("DOP") {
                                    if *min > data.obs {
                                        *min = data.obs ;
                                    }
                                    if *max < data.obs {
                                        *max = data.obs ;
                                    }
                                } else {
                                    y_min_max.insert("DOP".to_string(), (data.obs, data.obs));
                                }
                            } else {
                                if let Some((min,max)) = y_min_max.get_mut("SSI") {
                                    if *min > data.obs {
                                        *min = data.obs ;
                                    }
                                    if *max < data.obs {
                                        *max = data.obs ;
                                    }
                                } else {
                                    y_min_max.insert("SSI".to_string(), (data.obs, data.obs));
                                }
                            }
                        }
                    }
                }

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

/// Execute `teqc` ops on a single file
fn run_single_file_teqc_op (rnx: &Rinex, matches: clap::ArgMatches) {
    let ascii_plot = matches.is_present("ascii-plot");
    let _split = matches.is_present("split");
    let _split_epoch : Option<Epoch> = match matches.value_of("split") {
        Some(s) => {
            if let Ok(e) = parser::parse_epoch(s) {
                Some(e)
            } else {
                None
            }
        },
        None => None,
    };
    if ascii_plot {
        println!("{}", ascii_plot::ascii_plot(ascii_plot::DEFAULT_X_WIDTH, &rnx, None));
    }
}

/// Execute user requests on two files
fn run_double_file_op (
    rnx_a: &Rinex, 
    rnx_b: &Rinex, 
    matches: clap::ArgMatches,
    _output: Option<&str>) 
{
    let pretty = matches.is_present("pretty");
    let merge = matches.is_present("merge");
    let diff = matches.is_present("diff");

    if diff {
        if let Ok(rnx) = rnx_a.observation_diff(rnx_b) {
            // print remaining record data
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rnx.record).unwrap())
            } else {
                println!("{}", serde_json::to_string(&rnx.record).unwrap())
            }
        } 
    }
    if merge {
        if let Ok(rnx_c) = rnx_a.merge(rnx_b) {
            if rnx_c.to_file("merge.rnx").is_err() {
                panic!("failed to generate new file");
            }
        } else {
            panic!("merge() failed");
        }
    }
}*/

fn resample (rnx: &mut Rinex, ops: Vec<&str>) {}
fn filter (rnx: &mut Rinex, ops: Vec<&str>) {}
fn record_analysis (rnx: &Rinex, pretty: bool) {}

pub fn main () -> Result<(), rinex::Error> {
    let cli = Cli::new();
    let mut rnx = Rinex::from_file(cli.input_filepath())?;
    let pretty = cli.pretty();

    if cli.resampling() { // resampling requested
        resample_record(&rnx, cli.resampling_ops());
    }
    
    if cli.retain() { // retain data of interest
        retain_filters(&mut rnx, cli.retain_ops());
    }
    
    if cli.filter() { // apply desired filters
        //filter(&rnx, cli.retain_ops());
    }
    
    // grab data of interest
    if cli.extract() {
        extract_data(&rnx, cli.extraction_ops(), pretty);
    } else {
        // no data of interest
        // => extract record
        record_analysis(&rnx, pretty);
    }

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

    ////////////////////////////////////////
    // Parse, filter, resample
    ////////////////////////////////////////
    for fp in &filepaths {
        let path = std::path::PathBuf::from(fp);
        let mut rinex = match path.exists() {
            true => {
                if let Ok(r) = Rinex::from_file(fp) {
                    r
                } else {
                    println!("Failed to parse file \"{}\"", fp); 
                    continue
                }
            },
            false => {
                println!("File \"{}\" does not exist", fp);
                continue
            },
        };
        resample_single_file(&mut rinex, matches.clone());
        apply_filters(&mut rinex, matches.clone());
        queue.push(rinex);
    }

    let mut target_is_single_file = true;
    // Merge is a 2=>1 op
    target_is_single_file &= !matches.is_present("merge");
    // Diff and DDiff are 2=>1 ops
    target_is_single_file &= !matches.is_present("diff");
    target_is_single_file &= !matches.is_present("ddiff");
    
    /////////////////////////////////////
    // ops that require only 1 file
    /////////////////////////////////////
    if target_is_single_file {
        for i in 0..queue.len() {
            let output = outputs.get(i);
            run_single_file_op(&queue[i], matches.clone(), output.copied());
            run_single_file_teqc_op(&queue[i], matches.clone());
        }
    }

    /////////////////////////////////////
    // ops that require 2 files
    /////////////////////////////////////
    if !target_is_single_file { // User requested a 2D op
        for i in 0..queue.len()/2 {
            let q_2p = &queue[i*2];
            let q_2p1 = &queue[i*2+1]; 
            let output = outputs.get(i);
            run_double_file_op(&q_2p, &q_2p1, matches.clone(), output.copied());
        }
    }

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
    Ok(())
}// main
