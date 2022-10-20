//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Based on crate <https://github.com/gwbres/rinex>     
//! Homepage: <https://github.com/gwbres/rinex-cli>

mod cli; // command line interface 
mod extract; // high level data
mod teqc; // `teqc` operations
mod plot; // plotting operations
mod retain; // record filtering 
mod filter; // record filtering
mod resampling; // record resampling

use rinex::{*,
    version::Version, 
    observation::Crinex,
};

use cli::Cli;
use extract::extract_data;
use retain::retain_filters;
use filter::apply_filters;
use resampling::record_resampling;

pub fn main() -> Result<(), rinex::Error> {
    let cli = Cli::new();
    let pretty = cli.pretty();

    let plot = cli.plot();
    let input_paths = cli.input_paths();
    let output_paths = cli.output_paths();

    // list of parsed and preprocessed RINEX files
    let mut queue: Vec<Rinex> = Vec::new();
    for fp in input_paths {
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
        queue.push(rnx);
    }
    
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

    // Analyze all extracted RINEX,
    //  or possibly post-analyze after pre-processing
    for (index, rnx) in queue.iter_mut().enumerate() {
        if cli.extract() {
            // extract data of interest
            extract_data(&rnx, cli.extraction_ops(), pretty);
        } else {
            // no data of interest
            if let Some(path) = output_paths.get(index) {
                // output path provided
                //  detect special CRINEX case
                let is_obs = rnx.is_observation_rinex();
                if path.contains(".21D") {
                    if !is_obs {
                        println!("CRNX compression only applies to Observation RINEX"); 
                    } else {
                        // ==> RNX2CRX1 
                        let mut context = rnx.header.obs.clone().unwrap();
                        context.crinex = Some(Crinex {
                            version: Version {
                                major: 1,
                                minor: 0,
                            },
                            prog: "rust-rnx".to_string(),
                            date: chrono::Utc::now().naive_utc(),
                        });
                        // convert
                        rnx.header = rnx.header
                            .with_observation_fields(context);
                    }
                } else if path.contains(".crx") {
                    if !is_obs {
                        println!("CRNX compression only applies to Observation RINEX"); 
                    } else {
                        // ==> RNX2CRX3 
                        let mut context = rnx.header.obs.clone().unwrap();
                        context.crinex = Some(Crinex {
                            version: Version {
                                major: 3,
                                minor: 0,
                            },
                            prog: "rust-rnx".to_string(),
                            date: chrono::Utc::now().naive_utc(),
                        });
                        // convert
                        rnx.header = rnx.header
                            .with_observation_fields(context);
                    }
                }
                if rnx.to_file(path).is_ok() {
                    println!("\"{}\" has been generated", path);
                } else {
                    println!("failed to generate \"{}\"", path);
                }
            } else {
                // no output path provided
                // ==> data visualization
                if plot { // graphical 
                    let dims = cli.plot_dimensions(); // create context
                    let mut ctx = plot::Context::new(dims, &rnx); 
                    plot::plot_rinex(&mut ctx, &rnx); 
                } else { // stream to stdout
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
