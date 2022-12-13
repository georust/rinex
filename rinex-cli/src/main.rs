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

use horrorshow::Template;

use cli::Cli;
use filter::{
    apply_filters,      // special filters, with cli options
    apply_gnss_filters, // filter out undesired constellations
};
use identification::basic_identification;
use resampling::record_resampling;
use retain::retain_filters;
use rinex::{merge::Merge, prelude::*, processing::*, split::Split};

pub mod fops; // file operation helpers
pub mod parser; // command line parsing utilities

use std::io::Write;

use fops::{filename, open_html_with_default_app};

// Returns path prefix for all products to be generated
// like report output, generate files..
fn product_prefix(fname: &str) -> String {
    env!("CARGO_MANIFEST_DIR").to_owned() + "/product/" + &filename(fname)
}

/*
 * Applies elevation mask, to non Navigation RINEX
 */
//fn apply_elevation_mask(rnx: &mut Rinex, sv_angles: ) {};

pub fn main() -> Result<(), rinex::Error> {
    let cli = Cli::new();
    let quiet = cli.quiet();
    let pretty = cli.pretty();
    let qc_only = cli.quality_check_only();
    let mut ctx = plot::Context::new();

    // input context
    //  primary RINEX (-fp)
    //  optionnal RINEX (-nav, )
    let fp = cli.input_path();
    let mut rnx = Rinex::from_file(fp)?;
    let mut nav_context = cli.nav_context();

    /*
     * Reference / Ground station position
     *  1. if manually given by user: preferred
     *  2. otherwise: use provided context
     */
    let mut ref_position = cli.ref_position();
    if ref_position.is_none() {
        if let Some(pos) = rnx.header.coords {
            ref_position = Some(pos);
        } else {
            if let Some(ref nav) = nav_context {
                if let Some(pos) = nav.header.coords {
                    ref_position = Some(pos);
                }
            }
        }
    }

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
     * Elevation mask special case
     *   applies to Navigation data directly
     *   applies inderiectly to primary file, in "augmented" mode
     */
    if let Some(mask) = cli.elevation_mask() {
        if let Some(ref nav) = nav_context {
            let sv_angles = nav.navigation_sat_angles(cli.ref_position());
            //apply_elevation_mask(rnx, sv_angles, cli.ref_position()); //TODO
        } else {
            // apply to primary rinex, if possible
            rnx.elevation_mask_mut(mask, cli.ref_position());
        }
    }


    /*
     * Observation RINEX:
     *  align phase origins at this point
     *  this allows easy GNSS combination and processing,
     *  gives more meaningful phase data plots..
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
    if cli.sv_epoch() && !qc_only {
        analysis::sv_epoch(&mut ctx, &rnx, &mut nav_context);
    }
    /*
     * Epoch histogram analysis
     */
    if cli.epoch_histogram() && !qc_only {
        analysis::epoch_histogram(&mut ctx, &rnx);
    }
    /*
     * DCB analysis requested
     */
    if cli.dcb() && !qc_only {
        let mut data = rnx.observation_phase_dcb();
        for (op, inner) in rnx.observation_pseudorange_dcb() {
            data.insert(op.clone(), inner.clone());
        }
        plot::plot_gnss_recombination(&mut ctx, "Differential Code Biases", "DBCs [n.a]", &data);
    }
    /*
     * Code Multipath analysis
     */
    if cli.multipath() && !qc_only {
        let data = rnx.observation_code_multipath();
        plot::plot_gnss_recombination(&mut ctx, "Code Multipath Biases", "MP [n.a]", &data);
    }
    /*
     * [GF] recombination visualization requested
     */
    if cli.gf_recombination() && !qc_only {
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
    if cli.wl_recombination() && !qc_only {
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
    if cli.nl_recombination() && !qc_only {
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
    if cli.mw_recombination() && !qc_only {
        let data = rnx.observation_mw_combinations();
        plot::plot_gnss_recombination(
            &mut ctx,
            "Melbourne-Wübbena signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
    }
    /*
     * MERGE
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
     * SPLIT
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
     * skyplot
     */
    let nav_provided = rnx.is_navigation_rinex() || nav_context.is_some();
    let skyplot = nav_provided && !qc_only;
    if skyplot {
        plot::skyplot(&mut ctx, &rnx, &nav_context, ref_position);
    }
    /*
     * Record analysis / visualization
     * analysis depends on the provided record type
     */
    if !qc_only {
        plot::plot_record(&cli, &mut ctx, &rnx, &nav_context);
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
        let report = QcReport::basic(&rnx, &nav_context);
        if cli.quality_check_separate() {
            let qc_absolute_path = product_prefix.to_owned() + "/qc.html";
            let mut qc_fd = std::fs::File::create(&qc_absolute_path)
                .expect(&format!("failed to create \"{}\"", &qc_absolute_path));
            write!(qc_fd, "{}", report.to_html()).expect("failed to generate QC summary report");
            println!("QC report generated: \"{}\"", &qc_absolute_path);
        } else {
            // append to current HTML content
            html.push_str("<div=\"qc-report\">\n");
            html.push_str(&report.to_inline_html().into_string().unwrap());
            html.push_str("</div>\n");
        }
        println!("Quality check summary added to html report");
    }
    write!(html_fd, "{}", html).expect(&format!("failed to write HTML content"));
    if !quiet {
        open_html_with_default_app(&html_absolute_path);
    }
    println!("HTML report \"{}\" generated", &html_absolute_path);

    Ok(())
} // main
