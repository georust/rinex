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
mod analysis; // basic analysis operations
mod identification; // high level identification/macros

use rinex::*;
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

    // RINEX data 
    let mut rnx = Rinex::from_file(fp)?;
    let mut nav_context = cli.nav_context();

    /*
     * Preprocessing, filtering, resampling..
     */
    if cli.resampling() { // resampling requested
        record_resampling(&mut rnx, cli.resampling_ops());
        if let Some(ref mut nav) = nav_context {
            record_resampling(nav, cli.resampling_ops());
        }
    }
    if cli.retain() { // retain data of interest
        retain_filters(&mut rnx, cli.retain_flags(), cli.retain_ops());
        if let Some(ref mut nav) = nav_context {
            retain_filters(nav, cli.retain_flags(), cli.retain_ops());
        }
    }
    if cli.filter() { // apply desired filters
        apply_filters(&mut rnx, cli.filter_ops());
        if let Some(ref mut nav) = nav_context {
            apply_filters(nav, cli.filter_ops());
        }
    }
 
    /*
     * Basic file identification
     */
    if cli.basic_identification() {
        basic_identification(&rnx, cli.identification_ops(), pretty);
        return Ok(());
    }
    
    /*
     * Basic analysis requested
     */
    if cli.sv_epoch() {
        analysis::sv_epoch::analyze(&rnx, &mut nav_context, cli.plot_dimensions());
        return Ok(());
    }

    /*
     * Phase diff analysis 
     */
    if cli.phase_diff() {
        let data = rnx.observation_phase_diff();
        let dims = cli.plot_dimensions();
        plot::differential::plot(dims, &data);
        return Ok(());
    }

    /*
     * Code diff analysis
     */
    if cli.code_diff() {
        /*let data = rnx.observation_code_diff();
        if plot {
            let dims = cli.plot_dimensions();
            plot::differential::plot(dims, &data);
        } else {
            println!("{:#?}", data);
        }*/
        return Ok(());
    }

    /*
     * Code Multipath analysis
     */
    if cli.multipath() {
        if let Some(nav) = nav_context {
            return Ok(());
        } else {
            panic!("--nav must be provided for code multipath analysis");
        }
    }
    
    // if output path was provided
    //  we dump resulting RINEX into a file
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
            return Ok(());
        } else {
            panic!("failed to generate \"{}\"", path);
        }
    } 
    
    /*
     * Record analysis / visualization
     */
    if plot {
        let dims = cli.plot_dimensions();
        let mut ctx = plot::record::Context::new(dims, &rnx); 
        plot::record::plot(&mut ctx, &rnx); 
    
    } else {
        if pretty {
            println!("{}", serde_json::to_string_pretty(&rnx.record).unwrap())
        } else {
            println!("{}", serde_json::to_string(&rnx.record).unwrap())
        }
    }
    Ok(())
}// main
