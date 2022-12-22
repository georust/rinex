//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Based on crate <https://github.com/gwbres/rinex>     
//! Homepage: <https://github.com/gwbres/rinex-cli>

mod analysis; // basic analysis
mod cli; // command line interface
mod context; // RINEX context
pub mod fops; // file operation helpers
mod identification; // high level identification/macros
pub mod parser;
mod plot; // plotting operations

mod preprocessing;
use preprocessing::preprocess;

use horrorshow::Template;
use rinex::{merge::Merge, processing::*, split::Split};

use cli::Cli;
pub use context::Context;
use identification::rinex_identification;
use plot::PlotContext;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use fops::open_html_with_default_app;
use std::io::Write;

pub fn main() -> Result<(), rinex::Error> {
    pretty_env_logger::init_timed();

    let cli = Cli::new();
    let mut ctx = Context::new(&cli);
    let mut plot_ctx = PlotContext::new();

    let quiet = cli.quiet();

    let qc_only = cli.quality_check_only();
    let qc = cli.quality_check() || qc_only;

    /*
     * Preprocessing
     */
    preprocess(&mut ctx, &cli);

    /*
      <!>    <!>     <!>      <!>      <!>      <!>      <!>
     * Observation RINEX:
     *  align phase origins at this point
     *  this allows easy GNSS combination and processing,
     *  gives more meaningful phase data plots..
      <!>    <!>     <!>      <!>      <!>      <!>      <!>
    ctx.primary_rinex.observation_align_phase_origins_mut();
     */

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
    if cli.sv_epoch() {
        info!("sv/epoch analysis");
        analysis::sv_epoch(&ctx, &mut plot_ctx);
    }
    /*
     * Epoch histogram analysis
     */
    if cli.epoch_histogram() {
        info!("epoch histogram analysis");
        analysis::epoch_histogram(&ctx, &mut plot_ctx);
    }
    /*
     * DCB analysis requested
     */
    if cli.dcb() {
        let mut data = ctx.primary_rinex.observation_phase_dcb();
        for (op, inner) in ctx.primary_rinex.observation_pseudorange_dcb() {
            data.insert(op.clone(), inner.clone());
        }
        plot::plot_gnss_recombination(
            &mut plot_ctx,
            "Differential Code Biases",
            "DBCs [n.a]",
            &data,
        );
        info!("--dcb analysis");
    }
    /*
     * Code Multipath analysis
     */
    if cli.multipath() {
        let data = ctx
            .primary_rinex
            .observation_align_phase_origins()
            .observation_code_multipath();
        plot::plot_gnss_recombination(
            &mut plot_ctx,
            "Code Multipath Biases",
            "Meters of delay",
            &data,
        );
        info!("--mp analysis");
    }
    /*
     * [GF] recombination visualization requested
     */
    if cli.gf_recombination() {
        let data = ctx.primary_rinex.observation_gf_combinations();
        plot::plot_gnss_recombination(
            &mut plot_ctx,
            "Geometry Free signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
        info!("--gf recombination");
    }
    /*
     * Ionospheric Delay Detector (graph)
     */
    if cli.iono_detector() {
        let data = ctx.primary_rinex.observation_iono_detector();
        plot::plot_iono_detector(&mut plot_ctx, &data);
        info!("--iono detector");
    }
    /*
     * [WL] recombination
     */
    if cli.wl_recombination() {
        let data = ctx.primary_rinex.observation_wl_combinations();
        plot::plot_gnss_recombination(
            &mut plot_ctx,
            "Wide Lane signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
        info!("--wl recombination");
    }
    /*
     * [NL] recombination
     */
    if cli.nl_recombination() {
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
    if cli.mw_recombination() {
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
     */
    if let Some(to_merge) = ctx.to_merge {
        info!("[merge] special mode");
        //TODO
        /*if cli.resampling() {
            record_resampling(&mut rnx_b, cli.resampling_ops());
        }
        if cli.retain() {
            retain_filters(&mut rnx_b, cli.retain_flags(), cli.retain_ops());
        }
        if cli.filter() {
            apply_filters(&mut rnx_b, cli.filter_ops());
        }
        */
        // [1] proceed to merge
        ctx.primary_rinex
            .merge_mut(&to_merge)
            .expect("`merge` operation failed");
        info!("merge both rinex files");
        // [2] generate new file
        fops::generate(&ctx.primary_rinex, "merged.rnx").expect("failed to generate merged file");
        // [*] stop here, special mode: no further analysis allowed
        return Ok(());
    }

    /*
     * SPLIT
     */
    if let Some(epoch) = cli.split() {
        let (a, b) = ctx
            .primary_rinex
            .split(epoch)
            .expect("failed to split primary rinex file");

        let name = format!("{}-{}.rnx", cli.input_path(), a.epochs()[0]);
        fops::generate(&a, &name).expect(&format!("failed to generate \"{}\"", name));

        let name = format!("{}-{}.rnx", cli.input_path(), b.epochs()[0]);
        fops::generate(&b, &name).expect(&format!("failed to generate \"{}\"", name));

        // [*] stop here, special mode: no further analysis allowed
        return Ok(());
    }

    /*
     * skyplot
     */
    let skyplot = (ctx.primary_rinex.is_navigation_rinex() || ctx.nav_rinex.is_some()) && !qc_only;
    if skyplot {
        plot::skyplot(&ctx, &mut plot_ctx);
        info!("sky view generated");
    }

    /*
     * CS Detector
     */
    if cli.cs_graph() {
        info!("cs detector");
        //let mut detector = CsDetector::default();
        //let cs = detector.cs_detection(&ctx.primary_rinex);
    }

    /*
     * Record analysis / visualization
     * analysis depends on the provided record type
     */
    if !qc_only {
        info!("record analysis");
        plot::plot_record(&ctx, &mut plot_ctx);
    }

    /*
     * Render HTML
     */
    let html_absolute_path = ctx.prefix.to_owned() + "/analysis.html";
    let mut html_fd = std::fs::File::create(&html_absolute_path)
        .expect(&format!("failed to create \"{}\"", &html_absolute_path));
    let mut html = plot_ctx.to_html(cli.tiny_html());
    /*
     * Quality Check summary
     */
    if qc {
        info!("qc mode");
        let report = QcReport::basic(&ctx.primary_rinex, &ctx.nav_rinex);

        if cli.quality_check_separate() {
            let qc_absolute_path = ctx.prefix.to_owned() + "/qc.html";
            let mut qc_fd = std::fs::File::create(&qc_absolute_path)
                .expect(&format!("failed to create \"{}\"", &qc_absolute_path));
            write!(qc_fd, "{}", report.to_html()).expect("failed to generate QC summary report");
            info!("qc summary report \"{}\" generated", &qc_absolute_path);
        } else {
            // append QC to global html
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

    info!("html report \"{}\" generated", &html_absolute_path);
    Ok(())
} // main
