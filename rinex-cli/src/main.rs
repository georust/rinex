//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Homepage: <https://github.com/georust/rinex-cli>

mod analysis; // basic analysis
mod cli; // command line interface
pub mod fops; // file operation helpers
mod identification; // high level identification/macros
mod plot; // plotting operations

mod preprocessing;
use preprocessing::preprocess;

//use horrorshow::Template;
use rinex::{
    merge::Merge,
    observation::{Combine, Dcb, IonoDelay, Mp},
    prelude::RinexType,
    prelude::*,
    split::Split,
};

use rinex_qc::*;

use cli::Cli;
use identification::rinex_identification;
use plot::PlotContext;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use fops::open_with_web_browser;
use sp3::{prelude::SP3, Merge as SP3Merge};
use std::io::Write;
use std::path::{Path, PathBuf};

/*
 * Workspace location is fixed to rinex-cli/product/$primary
 * at the moment
 */
fn workspace_path(ctx: &QcContext) -> PathBuf {
    let primary_stem: &str = ctx
        .primary_path()
        .file_stem()
        .expect("failed to determine Workspace")
        .to_str()
        .expect("failed to determine Workspace");
    /*
     * In case $FILENAME.RNX.gz gz compressed, we extract "$FILENAME".
     * Can use .file_name() once https://github.com/rust-lang/rust/issues/86319  is stabilized
     */
    let primary_stem: Vec<&str> = primary_stem.split(".").collect();

    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("workspace")
        .join(primary_stem[0])
}

/*
 * Helper to create the workspace, ie.: where all reports
 * get generated and saved.
 */
pub fn create_workspace(path: PathBuf) {
    std::fs::create_dir_all(&path).expect(&format!(
        "failed to create Workspace \"{}\": permission denied!",
        path.to_string_lossy()
    ));
}

/*
 * Appends file content, to data context being constructed
 */
fn load_rinex_file(path: &Path, ctx: &mut QcExtraData<Rinex>, ftype: RinexType) {
    let fullpath = path.to_string_lossy();
    let new = Rinex::from_file(&fullpath);
    if let Ok(new) = new {
        if new.header.rinex_type != ftype {
            let stem = path.file_stem().unwrap();
            error!("file \"{}\" is not {}", stem.to_string_lossy(), ftype);
        } else {
            if ctx.paths().is_empty() {
                // first file of this type, to be passed
                ctx.data = new.clone();
                ctx.paths.push(path.to_path_buf());
                trace!("{} : \"{}\"", ftype, fullpath);
            } else {
                // after that: we use merge() to create a blob
                let r = ctx.data_mut().merge_mut(&new);
                if r.is_ok() {
                    ctx.paths.push(path.to_path_buf());
                    trace!("{} : \"{}\"", ftype, fullpath);
                } else {
                    error!("failed to stack {} : \"{}\"", ftype, fullpath);
                }
            }
        }
    } else {
        error!(
            "failed to parse file \"{}\": {:?}",
            fullpath,
            new.err().unwrap()
        );
    }
}

/*
 * Appends SP3 file content, to data context being constructed
 */
fn load_sp3_file(path: &Path, ctx: &mut QcExtraData<SP3>) {
    let fullpath = path.to_string_lossy();
    let new = SP3::from_file(&fullpath);
    if let Ok(new) = new {
        if ctx.paths().is_empty() {
            // first file of this type, to be passed
            ctx.data = new.clone();
            ctx.paths.push(path.to_path_buf());
            trace!("SP3 : \"{}\"", fullpath);
        } else {
            // after that: we use merge() to create a blob
            let r = ctx.data_mut().merge_mut(&new);
            if r.is_ok() {
                ctx.paths.push(path.to_path_buf());
                trace!("SP3 : \"{}\"", fullpath); 
            } else {
                error!("failed to stack sp3 : \"{}\"", fullpath); 
            }
        }
    } else {
        error!(
            "failed to parse file \"{}\": {:?}",
            fullpath,
            new.err().unwrap()
        );
    }
}

use std::fs::ReadDir;

/*
 * Appends directory content, to data context being constructed
 */
fn load_rinex_directory(dir: ReadDir, ctx: &mut QcExtraData<Rinex>, ftype: RinexType) {
    for entry in dir {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                /* recursive loading */
                if let Ok(dir) = path.read_dir() {
                    load_rinex_directory(dir, ctx, ftype);
                }
            } else {
                load_rinex_file(&path, ctx, ftype);
            }
        }
    }
}

/*
 * Appends SP3 directory content, to data context being constructed
 */
fn load_sp3_directory(dir: ReadDir, ctx: &mut QcExtraData<SP3>) {
    for entry in dir {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                /* recursive loading */
                if let Ok(dir) = path.read_dir() {
                    load_sp3_directory(dir, ctx);
                }
            } else {
                load_sp3_file(&path, ctx);
            }
        }
    }
}

/*
 * Macro to build QcExtraData of a specific RINEX Type
 */
fn build_extra_rinex_data(
    cli: &Cli,
    cli_key: &str,
    ftype: RinexType,
) -> Option<QcExtraData<Rinex>> {
    if let Some(dir) = cli.data_dir(cli_key) {
        let mut ctx = QcExtraData::<Rinex>::default();
        load_rinex_directory(dir, &mut ctx, ftype);
        Some(ctx)
    } else if let Some(paths) = cli.data_paths(cli_key) {
        let mut ctx = QcExtraData::<Rinex>::default();
        for path in paths {
            load_rinex_file(Path::new(path), &mut ctx, ftype);
        }
        Some(ctx)
    } else {
        None // no files provided by user
    }
}

/*
 * Macro to build QcExtraData of SP3 Type
 */
fn build_extra_sp3_data(cli: &Cli) -> Option<QcExtraData<SP3>> {
    if let Some(dir) = cli.data_dir("sp3") {
        let mut ctx = QcExtraData::<SP3>::default();
        load_sp3_directory(dir, &mut ctx);
        Some(ctx)
    } else if let Some(paths) = cli.data_paths("sp3") {
        let mut ctx = QcExtraData::<SP3>::default();
        for path in paths {
            load_sp3_file(Path::new(path), &mut ctx);
        }
        Some(ctx)
    } else {
        None // no files provided by user
    }
}

/*
 * Creates File/Data context defined by user.
 * Regroups all provided files/folders,
 */
fn create_context(cli: &Cli) -> QcContext {
    QcContext {
        primary: {
            let fp = cli.input_path();
            let data = QcPrimaryData::from_file(fp);
            if let Ok(data) = data {
                data
            } else {
                panic!(
                    "failed to rinex file \"{}\" - {:?}",
                    fp,
                    data.err().unwrap()
                );
            }
        },
        nav: build_extra_rinex_data(cli, "nav", RinexType::NavigationData),
        atx: build_extra_rinex_data(cli, "atx", RinexType::AntennaData),
        sp3: build_extra_sp3_data(cli),
    }
}

/*
 * Returns true if Skyplot view if feasible
 */
fn skyplot_allowed(ctx: &QcContext, cli: &Cli) -> bool {
    if cli.quality_check_only() {
        /*
         * Special mode: no plots allowed
         */
        return false;
    }

    let has_nav = ctx.has_navigation_data();
    let has_ref_position = ctx.ground_position().is_some() || cli.manual_position().is_some();
    if has_nav && !has_ref_position {
        info!("missing a reference position for the skyplot view.");
        info!("see rinex-cli -h : antenna positions.");
    }
    has_nav && has_ref_position
}

pub fn main() -> Result<(), rinex::Error> {
    pretty_env_logger::init_timed();

    // Cli
    let cli = Cli::new();
    let quiet = cli.quiet();
    let qc_only = cli.quality_check_only();
    let qc = cli.quality_check() || qc_only;

    // Initiate plot context
    let mut plot_ctx = PlotContext::new();

    // Initiate QC parameters
    let mut qc_opts = cli.qc_config();

    // Build file context
    let mut ctx = create_context(&cli);

    // Workspace
    let workspace = workspace_path(&ctx);
    info!("workspace is \"{}\"", workspace.to_string_lossy());
    create_workspace(workspace.clone());

    /*
     * Emphasize which reference position is to be used.
     * This will help user make sure everything is correct.
     * [+] Cli: always superceeds
     * [+] then QC config file is prefered (advanced usage)
     * [+] eventually we rely on the context pool.
     * Missing ref. position may restrict possible operations.
     */
    if let Some(pos) = cli.manual_position() {
        info!("using manually defined reference position {}", pos);
    } else if let Some(pos) = ctx.ground_position() {
        info!("using reference position {}", pos);
    } else {
        info!("no reference position given or identified");
    }

    /*
     * Preprocessing
     */
    preprocess(&mut ctx, &cli);

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
    if cli.sampling_histogram() {
        info!("sample rate histogram analysis");
        analysis::sampling::histogram(&ctx, &mut plot_ctx);
    }
    /*
     * DCB analysis requested
     */
    if cli.dcb() {
        let data = ctx
            .primary_data()
            .observation_phase_align_origin()
            .observation_phase_carrier_cycles()
            .dcb();
        plot::plot_gnss_dcb(
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
        //let data = ctx
        //    .primary_data()
        //    .observation_phase_align_origin()
        //    .observation_phase_carrier_cycles()
        //    .mp();
        //plot::plot_gnss_dcb(
        //    &mut plot_ctx,
        //    "Code Multipath Biases",
        //    "Meters of delay",
        //    &data,
        //);
        warn!("--mp analysis not available yet");
    }
    /*
     * [GF] recombination visualization requested
     */
    if cli.gf_recombination() {
        let data = ctx
            .primary_data()
            .observation_phase_align_origin()
            .geo_free();
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
        let data = ctx.primary_data().iono_delay(Duration::from_seconds(360.0));

        plot::plot_iono_detector(&mut plot_ctx, &data);
        info!("--iono detector");
    }
    /*
     * [WL] recombination
     */
    if cli.wl_recombination() {
        let data = ctx.primary_data().wide_lane();
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
        let data = ctx.primary_data().narrow_lane();
        plot::plot_gnss_recombination(
            &mut plot_ctx,
            "Narrow Lane signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
        info!("--nl recombination");
    }
    /*
     * [MW] recombination
     */
    if cli.mw_recombination() {
        let data = ctx.primary_data().melbourne_wubbena();
        plot::plot_gnss_recombination(
            &mut plot_ctx,
            "Melbourne-Wübbena signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
        info!("--mw recombination");
    }
    /*
     * MERGE
     */
    if let Some(rinex_b) = cli.to_merge() {
        info!("merging files..");

        // [1] proceed to merge
        let new_rinex = ctx
            .primary_data()
            .merge(&rinex_b)
            .expect("merging operation failed");

        //TODO: make this path programmable
        let path = workspace.clone().join("merged.rnx");

        let path = path
            .as_path()
            .to_str()
            .expect("failed to generate merged file");

        // [2] generate new file
        new_rinex
            .to_file(path)
            .expect("failed to generate merged file");

        info!("\"{}\" has been generated", &path);
        return Ok(());
    }
    /*
     * SPLIT
     */
    if let Some(epoch) = cli.split() {
        let (rnx_a, rnx_b) = ctx
            .primary_data()
            .split(epoch)
            .expect("failed to split primary rinex file");

        let file_stem = ctx
            .primary_path()
            .file_stem()
            .expect("failed to determine file prefix")
            .to_str()
            .expect("failed to determine file prefix");

        let file_suffix = rnx_a
            .first_epoch()
            .expect("failed to determine file suffix")
            .to_string();

        let path = format!(
            "{}/{}-{}.txt",
            workspace.to_string_lossy(),
            file_stem,
            file_suffix
        );

        rnx_a
            .to_file(&path)
            .expect(&format!("failed to generate splitted file \"{}\"", path));

        let file_suffix = rnx_b
            .first_epoch()
            .expect("failed to determine file suffix")
            .to_string();

        let path = format!(
            "{}/{}-{}.txt",
            workspace.to_string_lossy(),
            file_stem,
            file_suffix
        );

        rnx_b
            .to_file(&path)
            .expect(&format!("failed to generate splitted file \"{}\"", path));

        // [*] stop here, special mode: no further analysis allowed
        return Ok(());
    }
    /*
     * skyplot
     */
    if skyplot_allowed(&ctx, &cli) {
        plot::skyplot(&ctx, &mut plot_ctx);
        info!("skyplot view generated");
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
        info!("entering record analysis");
        plot::plot_record(&ctx, &mut plot_ctx);
    }
    /*
     * Render Graphs (HTML)
     */
    if !qc_only {
        let html_path = workspace_path(&ctx).join("graphs.html");
        let html_path = html_path.to_str().unwrap();

        let mut html_fd = std::fs::File::create(html_path)
            .expect(&format!("failed to create \"{}\"", &html_path));
        write!(html_fd, "{}", plot_ctx.to_html()).expect("failed to render graphs");

        info!("graphs rendered in $WORKSPACE/graphs.html");
        if !quiet {
            open_with_web_browser(&html_path);
        }
    }

    /*
     * QC Mode
     */
    if qc {
        info!("entering qc mode");
        /*
         * QC Config / versus command line
         * let the possibility to define some parameters
         * from the command line, instead of the config file.
         */
        if qc_opts.ground_position.is_none() {
            // config did not specify it
            if let Some(pos) = cli.manual_position() {
                // manually passed
                qc_opts = qc_opts.with_ground_position_ecef(pos.to_ecef_wgs84());
            }
        }

        /*
         * Print some info about current setup & parameters, prior analysis.
         */
        info!("Classification method : {:?}", qc_opts.classification);
        info!("Reference position    : {:?}", qc_opts.ground_position);
        info!("Minimal SNR           : {:?}", qc_opts.min_snr_db);
        info!("Elevation mask        : {:?}", qc_opts.elev_mask);
        info!("Sampling gap tolerance: {:?}", qc_opts.gap_tolerance);

        let html_report = QcReport::html(ctx, qc_opts);

        let report_path = workspace.join("report.html");
        let mut report_fd = std::fs::File::create(&report_path).expect(&format!(
            "failed to create report \"{}\" : permission denied",
            report_path.to_string_lossy()
        ));

        write!(report_fd, "{}", html_report).expect("failed to generate QC summary report");

        info!("qc report $WORSPACE/report.html has been generated");
        if !quiet {
            open_with_web_browser(&report_path.to_string_lossy());
        }
    }
    Ok(())
} // main
