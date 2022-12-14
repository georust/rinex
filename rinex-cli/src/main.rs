//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Based on crate <https://github.com/gwbres/rinex>     
//! Homepage: <https://github.com/gwbres/rinex-cli>

mod cli; // command line interface
mod context; // RINEX context
mod analysis; // basic analysis
mod file_generation;
mod filter; // record filtering
mod identification; // high level identification/macros
mod plot; // plotting operations
mod resampling; // record resampling
mod retain; // record filtering
pub mod fops; // file operation helpers
pub mod parser; // command line parsing utilities

use horrorshow::Template;
use rinex::{merge::Merge, prelude::*, processing::*, split::Split};

use cli::Cli;
use filter::{
    apply_filters,      // special filters, with cli options
    apply_gnss_filters, // filter out undesired constellations
    elevation_mask_filter, // apply given elevation mask
};
pub use context::Context;
use plot::PlotContext;

// preprocessing
use resampling::resampling;
use retain::retain_filters;
use identification::rinex_identification;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use std::io::Write;
use fops::{filename, open_html_with_default_app};

/*
 * Applies elevation mask, to non Navigation RINEX
 */
//fn apply_elevation_mask(rnx: &mut Rinex, sv_angles: ) {};

pub fn main() -> Result<(), rinex::Error> {
    pretty_env_logger::init_timed();

    let cli = Cli::new();
    let mut ctx = Context::new(&cli);
    let mut plot_ctx = PlotContext::new();

    let quiet = cli.quiet();
    let pretty = cli.pretty();
    let qc_only = cli.quality_check_only();

    /*
     * Preprocessing, filtering, resampling..
     */
    if cli.resampling() {
        resampling(&mut ctx, &cli);
    }
    if cli.retain() {
        retain_filters(&mut ctx, &cli);
    }
    if cli.filter() {
        apply_gnss_filters(&mut ctx, &cli);
        apply_filters(&mut ctx, &cli);
        elevation_mask_filter(&mut ctx, &cli);
    }
    
    /*
     * Observation RINEX:
     *  align phase origins at this point
     *  this allows easy GNSS combination and processing,
     *  gives more meaningful phase data plots..
     */
    ctx.primary_rinex.observation_align_phase_origins_mut();

    /*
     * Basic file identification
     */
    if cli.identification() {
        rinex_identification(&ctx, &cli);
        return Ok(()); // not proceeding further, in this mode
    }
    
    /*
     * SV per Epoch analysis requested
     */
    if cli.sv_epoch() && !qc_only {
        info!("sv/epoch analysis");
        analysis::sv_epoch(&ctx, &mut plot_ctx);
    }
    /*
     * Epoch histogram analysis
     */
    if cli.epoch_histogram() && !qc_only {
        info!("epoch histogram analysis");
        analysis::epoch_histogram(&ctx, &mut plot_ctx);
    }
    /*
     * DCB analysis requested
     */
    if cli.dcb() && !qc_only {
        let mut data = ctx.primary_rinex.observation_phase_dcb();
        for (op, inner) in ctx.primary_rinex.observation_pseudorange_dcb() {
            data.insert(op.clone(), inner.clone());
        }
        plot::plot_gnss_recombination(&mut plot_ctx, "Differential Code Biases", "DBCs [n.a]", &data);
        info!("dcb analysis generated");
    }
    /*
     * Code Multipath analysis
     */
    if cli.multipath() && !qc_only {
        let data = ctx.primary_rinex.observation_code_multipath();
        plot::plot_gnss_recombination(&mut plot_ctx, "Code Multipath Biases", "MP [n.a]", &data);
        info!("mp analysis generated");
    }
    /*
     * [GF] recombination visualization requested
     */
    if cli.gf_recombination() && !qc_only {
        let data = ctx.primary_rinex.observation_gf_combinations();
        plot::plot_gnss_recombination(
            &mut plot_ctx,
            "Geometry Free signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
        info!("gf recombination");
    }
    /*
     * [WL] recombination
     */
    if cli.wl_recombination() && !qc_only {
        let data = ctx.primary_rinex.observation_wl_combinations();
        plot::plot_gnss_recombination(
            &mut plot_ctx,
            "Wide Lane signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
        info!("wl recombination");
    }
    /*
     * [NL] recombination
     */
    if cli.nl_recombination() && !qc_only {
        let data = ctx.primary_rinex.observation_nl_combinations();
        plot::plot_gnss_recombination(
            &mut plot_ctx,
            "Narrow Lane signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
        info!("nl recombination");
    }
    /*
     * [MW] recombination
     */
    if cli.mw_recombination() && !qc_only {
        let data = ctx.primary_rinex.observation_mw_combinations();
        plot::plot_gnss_recombination(
            &mut plot_ctx,
            "Melbourne-Wübbena signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
        info!("mw recombination");
    }
    /*
     * MERGE
    
    if let Some(b_path) = cli.merge() {
        info!("[merge] special mode");
        // we're merging (A)+(B) into (C)
        let mut rnx_b = Rinex::from_file(b_path)?;
        info!("parsed \"{}\" correctly", filename(b_path));
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
     */
    /*
     * SPLIT
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
    */
    /*
     * skyplot
     */
    let skyplot = (ctx.primary_rinex.is_navigation_rinex() || ctx.nav_rinex.is_some()) && !qc_only;
    if skyplot {
        plot::skyplot(&ctx, &mut plot_ctx);
        info!("sky view generated");
    }
    /*
    /*
     * Record analysis / visualization
     * analysis depends on the provided record type
     */
    if !qc_only {
        plot::plot_record(&cli, &mut ctx, &rnx, &nav_context);
        info!("record analysis generated");
    }
    /*
     * Render HTML
     */
    let html_absolute_path = product_prefix.to_owned() + "/analysis.html";
    let mut html_fd = std::fs::File::create(&html_absolute_path)
        .expect(&format!("failed to create \"{}\"", &html_absolute_path));
    let mut html = ctx.to_html(cli.tiny_html());
    /*
     * Quality Check summary
     */
    if cli.quality_check() {
        info!("entering qc mode");
        let report = QcReport::basic(&rnx, &nav_context);
        if cli.quality_check_separate() {
            let qc_absolute_path = product_prefix.to_owned() + "/qc.html";
            let mut qc_fd = std::fs::File::create(&qc_absolute_path)
                .expect(&format!("failed to create \"{}\"", &qc_absolute_path));
            write!(qc_fd, "{}", report.to_html()).expect("failed to generate QC summary report");
            info!("qc summary report \"{}\" generated", &qc_absolute_path);
        } else {
            // append to current HTML content
            html.push_str("<div=\"qc-report\">\n");
            html.push_str(&report.to_inline_html().into_string().unwrap());
            html.push_str("</div>\n");
            info!("qc summary added to html report");
        }
    }
    write!(html_fd, "{}", html).expect(&format!("failed to write HTML content"));
    if !quiet {
        open_html_with_default_app(&html_absolute_path);
    }
    info!("html report \"{}\" generated", &html_absolute_path);*/
    Ok(())
} // main
