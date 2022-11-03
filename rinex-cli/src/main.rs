//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Based on crate <https://github.com/gwbres/rinex>     
//! Homepage: <https://github.com/gwbres/rinex-cli>

mod cli; // command line interface 
mod teqc; // `teqc` operations
mod plot; // plotting operations
mod retain; // record filtering 
mod filter; // record filtering
mod resampling; // record resampling
mod identification; // high level identification/macros

use rinex::{*,
    version::Version, 
    observation::Crinex,
};

use cli::Cli;
use retain::retain_filters;
use filter::apply_filters;
use resampling::record_resampling;
use identification::basic_identification;

pub fn main() -> Result<(), rinex::Error> {
    let cli = Cli::new();
    let pretty = cli.pretty();

    let plot = cli.plot();
    let fp = cli.input_path();
    let output_path = cli.output_path();

    // list of parsed and preprocessed RINEX files
    let mut rnx = Rinex::from_file(fp)?;

    if cli.resampling() { // resampling requested
        record_resampling(&mut rnx, cli.resampling_ops());
    }
    if cli.retain() { // retain data of interest
        retain_filters(&mut rnx, cli.retain_flags(), cli.retain_ops());
    }
    if cli.filter() { // apply desired filters
        apply_filters(&mut rnx, cli.filter_ops());
    }
 
/*
    // perform desired processing, if any
    if let Some(ref_rinex) = cli.single_diff() {
        let offset = queue.len();
        // perform 1D diff with given Reference Observations
        // on every provided RINEX files
        for index in 0..offset {
            let base_rnx = &queue[index];
            // compute 1D diff and push into queue
            if let Ok(rnx) = ref_rinex.observation_diff(&base_rnx) {
                queue.push(rnx);
            } else {
                println!("one differentiation operation failed");
            }
        }
        // retain Reference RINEX only
        // this allows post processing analysis on resulting data
        queue.drain(0..offset);
    }

    if let Some(context) = cli.double_diff() {
        // Double differentiation
        let ref_header = &context
            .observations
            .header;
        let offset = queue.len();
        for index in 0..offset {
            let base_rnx = &queue[index];
            // compute 2D diff and push into queue
            if let Ok(record) = context.double_diff(&base_rnx) {
                queue.push(Rinex::default()
                    .with_header(ref_header.clone())
                    .with_record(record::Record::ObsRecord(record)));
            } else {
                println!("one 2D differentiation operation failed");
            }
        }
        // retain Reference RINEX only
        // this allows post processing analysis on resulting data
        queue.drain(0..offset);
    }
*/
    if cli.basic_identification() {
        // basic RINEX identification requested
        basic_identification(&rnx, cli.identification_ops(), pretty);
    
    } else {
        // no basic ID
        //  ==> record study or advanced analysis
        if let Some(path) = output_path {
            // output path provided
            //  detect special CRINEX case
            let is_obs = rnx.is_observation_rinex();
            if path.contains(".21D") {
                if !is_obs {
                    println!("CRNX compression only applies to Observation RINEX"); 
                } else {
                    rnx.rnx2crnx();
                }
            } else if path.contains(".crx") {
                if !is_obs {
                    println!("CRNX compression only applies to Observation RINEX"); 
                } else {
                    rnx.rnx2crnx();
                }
            }
            if rnx.to_file(&path).is_ok() {
                println!("\"{}\" has been generated", path);
            } else {
                println!("failed to generate \"{}\"", path);
            }
        } else {
            // no output path provided
            // ==> data visualization
            if plot { // graphical 
                let dims = cli.plot_dimensions(); // create context
                
                if cli.phase_diff() {
                    // Differential phase code analysis
                    let mut ctx = plot::differential::Context::new(dims, &rnx); 
                    let data = rnx.observation_phase_diff();
                    plot::differential::plot(&mut ctx, data);
                
                } else if cli.code_diff() {
                    // Differential pseudo code analysis
                    let mut ctx = plot::differential::Context::new(dims, &rnx); 
                    let data = rnx.observation_phase_diff();
                    plot::differential::plot(&mut ctx, data);

                } else { // plot record
                    let mut ctx = plot::record::Context::new(dims, &rnx); 
                    plot::record::plot(&mut ctx, &rnx); 
                }

            } else { // stream to stdout
                if cli.phase_diff() {
                    // Differential phase code analysis
                    let data = rnx.observation_phase_diff();
                    println!("{:#?}", data);

                } else if cli.code_diff() {
                    // Differential pseudo code analysis

                } else {
                    // Full record display
                    if pretty {
                        println!("{}", serde_json::to_string_pretty(&rnx.record).unwrap())
                    } else {
                        println!("{}", serde_json::to_string(&rnx.record).unwrap())
                    }
                }
            }
        }
    }
    Ok(())
}// main
