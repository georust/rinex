//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Based on crate <https://github.com/gwbres/rinex>     
//! Homepage: <https://github.com/gwbres/rinex-cli>

mod analysis; // basic analysis operations
mod cli; // command line interface
mod file_generation;
mod filter; // record filtering
mod identification; // high level identification/macros
mod plot; // plotting operations
mod resampling; // record resampling
mod retain; // record filtering
mod teqc; // `teqc` operations // RINEX to file macro

use cli::Cli;
use filter::{
    apply_filters,      // special filters, with cli options
    apply_gnss_filters, // filter out undesired constellations
};
use identification::basic_identification;
use resampling::record_resampling;
use retain::retain_filters;
use rinex::{merge::Merge, split::Split, *};
use teqc::summary_report;

pub mod fops; // file operation helpers
pub mod parser; // command line parsing utilities

use std::io::Write;

use fops::{filename, open_html_with_default_app};

// Returns path prefix for all products to be generated
// like report output, generate files..
fn product_prefix(fname: &str) -> String {
    env!("CARGO_MANIFEST_DIR").to_owned() + "/product/" + &filename(fname)
}

pub fn main() -> Result<(), rinex::Error> {
    let cli = Cli::new();
    let pretty = cli.pretty();
    let mut ctx = plot::Context::new();

    // input context
    //  primary RINEX (-fp)
    //  optionnal RINEX (-nav, )
    let fp = cli.input_path();
    let mut rnx = Rinex::from_file(fp)?;
    let mut nav_context = cli.nav_context();
    let ref_position = cli.ref_position();

    // create subdirs we might need when studying this context
    let short_fp = filename(fp);
    let product_prefix = product_prefix(&short_fp);
    let _ = std::fs::create_dir_all(&product_prefix);

    /*
     * Preprocessing, filtering, resampling..
     */
    if cli.resampling() {
        // resampling requested
        record_resampling(&mut rnx, cli.resampling_ops());
        if let Some(ref mut nav) = nav_context {
            record_resampling(nav, cli.resampling_ops());
        }
    }
    if cli.retain() {
        // retain data of interest
        retain_filters(&mut rnx, cli.retain_flags(), cli.retain_ops());
        if let Some(ref mut nav) = nav_context {
            retain_filters(nav, cli.retain_flags(), cli.retain_ops());
        }
    }
    if cli.filter() {
        // apply desired filters
        //GNSS filter: get rid of undesired constellations
        apply_gnss_filters(&cli, &mut rnx);
        if let Some(ref mut nav) = nav_context {
            apply_gnss_filters(&cli, nav);
        }
        //Filters: get rid of undesired data sets
        apply_filters(&mut rnx, cli.filter_ops());
        if let Some(ref mut nav) = nav_context {
            apply_filters(nav, cli.filter_ops());
        }
    }

    /*
     * Observation RINEX:
     *  align phase origins at this point
     *  this allows easy GNSS combination and processing,
     *  also gives a more meaningful record plot
     */
    rnx.observation_align_phase_origins_mut();

    /*
     * Basic file identification
     */
    if cli.basic_identification() {
        basic_identification(&rnx, cli.identification_ops(), pretty);
        return Ok(());
    }

    /*
     * SV per Epoch analysis requested
     */
    if cli.sv_epoch() {
        analysis::sv_epoch(&mut ctx, &rnx, &mut nav_context);
    }
    /*
     * Epoch histogram analysis
     */
    if cli.epoch_histogram() {
        analysis::epoch_histogram(&mut ctx, &rnx);
    }

    /*
     * DCB analysis requested
     */
    if cli.dcb() {
        let mut data = rnx.observation_phase_dcb();
        for (op, inner) in rnx.observation_pseudorange_dcb() {
            data.insert(op.clone(), inner.clone());
        }
        plot::plot_gnss_recombination(&mut ctx, "Differential Code Biases", "DBCs [n.a]", &data);
    }

    /*
     * Code Multipath analysis
     */
    if cli.multipath() {
        let data = rnx.observation_code_multipath();
        plot::plot_gnss_recombination(&mut ctx, "Code Multipath Biases", "MP [n.a]", &data);
    }
    /*
     * [GF] recombination visualization requested
     */
    if cli.gf_recombination() {
        let data = rnx.observation_gf_combinations();
        plot::plot_gnss_recombination(
            &mut ctx,
            "Geometry Free signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
    }
    /*
     * [WL] recombination
     */
    if cli.wl_recombination() {
        let data = rnx.observation_wl_combinations();
        plot::plot_gnss_recombination(
            &mut ctx,
            "Wide Lane signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
    }
    /*
     * [NL] recombination
     */
    if cli.nl_recombination() {
        let data = rnx.observation_nl_combinations();
        plot::plot_gnss_recombination(
            &mut ctx,
            "Narrow Lane signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
    }
    /*
     * [MW] recombination
     */
    if cli.mw_recombination() {
        let data = rnx.observation_mw_combinations();
        plot::plot_gnss_recombination(
            &mut ctx,
            "Melbourne-Wübbena signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
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
        rnx.merge_mut(&rnx_b).expect("`merge` operation failed");
        // [2] generate new file
        file_generation::generate(&rnx, cli.output_path(), "merged.rnx").unwrap();
        // [*] stop here, special mode: no further analysis allowed
        return Ok(());
    }

    /*
     * teqc [SPLIT]
     */
    if let Some(epoch) = cli.split() {
        let (a, b) = rnx
            .split(epoch)
            .expect(&format!("failed to split \"{}\" into two", fp));

        let name = format!("{}-{}.rnx", cli.input_path(), a.epochs()[0]);
        file_generation::generate(&a, None, &name).unwrap();

        let name = format!("{}-{}.rnx", cli.input_path(), b.epochs()[0]);
        file_generation::generate(&b, None, &name).unwrap();

        // [*] stop here, special mode: no further analysis allowed
        return Ok(());
    }

    /*
     * TEQC `qc`
     */
    if cli.qc() {
        summary_report(&cli, &product_prefix, &rnx, &nav_context);
        return Ok(()); // special mode,
                       // runs quicker for users that are only interested in `-qc`
    }

    /*
     * skyplot view
     */
    let skyplot = rnx.is_navigation_rinex() || nav_context.is_some();
    if skyplot {
        plot::skyplot(&mut ctx, &rnx, &nav_context, ref_position);
    }

    /*
     * Record analysis / visualization
     */
    plot::plot_record(&mut ctx, &rnx, &nav_context);

    // Render HTML
    let html_absolute_path = product_prefix.to_owned() + "/analysis.html";
    let mut html_fd = std::fs::File::create(&html_absolute_path)
        .expect(&format!("failed to create \"{}\"", &html_absolute_path));
    let html = ctx.to_html(cli.tiny_html());
    write!(html_fd, "{}", html).expect(&format!("failed to write HTML content"));
    open_html_with_default_app(&html_absolute_path);
    println!("\"{}\" generated", &html_absolute_path);

    Ok(())
} // main
