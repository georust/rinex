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
mod file_generation; // RINEX to file macro

use rinex::*;
use cli::Cli;
use retain::retain_filters;
use filter::apply_filters;
use resampling::record_resampling;
use identification::basic_identification;

pub mod parser; // command line parsing utilities

pub fn main() -> Result<(), rinex::Error> {
    let cli = Cli::new();
    let pretty = cli.pretty();

    let plot = cli.plot();
    let fp = cli.input_path();

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
        plot::differential::plot(dims, 
            "phase-diff.png", 
            "PH Code Differential analysis",
            "Phase Difference [n.a]",
            &data);
        return Ok(());
    }

    /*
     * PR diff analysis
     */
    if cli.pseudorange_diff() {
        let data = rnx.observation_pseudorange_diff();
        let dims = cli.plot_dimensions();
        plot::differential::plot(dims, 
            "pseudorange-diff.png", 
            "PR Code Differential analysis",
            "PR Difference [n.a]",
            &data);
        return Ok(());
    }

    /*
     * PH + PR diff efficient impl
     */
    if cli.code_diff() {
        let (ph_data, pr_data) = rnx.observation_code_diff();
        let dims = cli.plot_dimensions();
        plot::differential::plot(dims, 
            "phase-diff.png", 
            "PH Code Differential analysis",
            "Phase Difference [n.a]",
            &ph_data);
        plot::differential::plot(dims, 
            "pseudorange-diff.png", 
            "PR Code Differential analysis",
            "PR Difference [n.a]",
            &pr_data);
    }

    /*
     * Code Multipath analysis
     */
    if cli.multipath() {
        /*if let Some(nav) = nav_context {
            return Ok(());
        } else {
            panic!("--nav must be provided for code multipath analysis");
        }*/
        panic!("code multipath analysis is under development");
    }
    
    /*
     * teqc [MERGE]
     */
    if let Some(rnx_b) = cli.merge() {
        // we're merging (A)+(B) into (C)
        let mut rnx_b = Rinex::from_file(rnx_b)?;
        // [0] apply similar conditions to RNX_b
        if cli.resampling() {
            record_resampling(&mut rnx_b, cli.resampling_ops());
        }
        if cli.retain() {
            retain_filters(&mut rnx_b, cli.retain_flags(), cli.retain_ops());
        }
        if cli.filter() {
            apply_filters(&mut rnx_b, cli.filter_ops());
        }
        // [1] proceed to merge
        rnx.merge_mut(&rnx_b)
            .expect("`merge` operation failed");
        // [2] generate new file
        file_generation::generate(&rnx, cli.output_path(), "merged.rnx")
            .unwrap();
        // [*] stop here, special mode: no further analysis allowed
        return Ok(());
    }
    
    /*
     * teqc [SPLIT]
     */
    if let Some(epoch) = cli.split() {
        let (a,b) = rnx.split(epoch)
            .expect(&format!("failed to split \"{}\" into two", fp));

        let first_date = a.epochs()[0].date;
        let name = format!("{}.rnx", first_date);
        file_generation::generate(&a, None, &name)
            .unwrap();
        
        let first_date = b.epochs()[0].date;
        let name = format!("{}.rnx", first_date);
        file_generation::generate(&b, None, &name)
            .unwrap();
        
        // [*] stop here, special mode: no further analysis allowed
        return Ok(());
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
