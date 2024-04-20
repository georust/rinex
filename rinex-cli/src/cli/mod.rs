use log::info;
use std::{
    fs::create_dir_all,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::{value_parser, Arg, ArgAction, ArgMatches, ColorChoice, Command};
use rinex::prelude::*;

use crate::fops::open_with_web_browser;

// identification mode
mod identify;
// graph mode
mod graph;
// QC mode
mod qc;
// positioning mode
mod positioning;

// file operations
mod fops;

use fops::{filegen, merge, split, substract, time_binning};

pub struct Cli {
    /// Arguments passed by user
    pub matches: ArgMatches,
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}

/// Context defined by User.
pub struct Context {
    /// Data context defined by user
    pub data: RnxContext,
    /// Quiet option
    pub quiet: bool,
    /// Workspace is the place where this session will generate data.
    /// By default it is set to $WORKSPACE/$PRIMARYFILE.
    /// $WORKSPACE is either manually definedd by CLI or we create it (as is).
    /// $PRIMARYFILE is determined from the most major file contained in the dataset.
    pub workspace: PathBuf,
    /// Context name is derived from the primary file loaded in Self,
    /// and mostly used in session products generation.
    pub name: String,
    /// (RX) reference position to be used in further analysis.
    /// It is either (priority order is important)
    ///  1. manually defined by CLI
    ///  2. determined from dataset
    pub rx_ecef: Option<(f64, f64, f64)>,
}

impl Context {
    /*
     * Utility to determine the most major filename stem,
     * to be used as the session workspace
     */
    pub fn context_stem(data: &RnxContext) -> String {
        let ctx_major_stem: &str = data
            .primary_path()
            .expect("failed to determine a context name")
            .file_stem()
            .expect("failed to determine a context name")
            .to_str()
            .expect("failed to determine a context name");
        /*
         * In case $FILENAME.RNX.gz gz compressed, we extract "$FILENAME".
         * Can use .file_name() once https://github.com/rust-lang/rust/issues/86319  is stabilized
         */
        let primary_stem: Vec<&str> = ctx_major_stem.split('.').collect();
        primary_stem[0].to_string()
    }
    /*
     * Utility to prepare subdirectories in the session workspace
     */
    pub fn create_subdir(&self, suffix: &str) {
        create_dir_all(self.workspace.join(suffix))
            .unwrap_or_else(|e| panic!("failed to generate session dir {}: {:?}", suffix, e));
    }
    /*
     * Utility to create a file in this session
     */
    fn create_file(&self, path: &Path) -> std::fs::File {
        std::fs::File::create(path).unwrap_or_else(|e| {
            panic!("failed to create {}: {:?}", path.display(), e);
        })
    }
    /*
     * Save HTML content, auto opens it if quiet (-q) is not turned on
     */
    pub fn render_html(&self, filename: &str, html: String) {
        let path = self.workspace.join(filename);
        let mut fd = self.create_file(&path);
        write!(fd, "{}", html).unwrap_or_else(|e| {
            panic!("failed to render HTML content: {:?}", e);
        });
        info!("html rendered in \"{}\"", path.display());

        if !self.quiet {
            open_with_web_browser(path.to_string_lossy().as_ref());
        }
    }
}

impl Cli {
    /// Build new command line interface
    pub fn new() -> Self {
        Self {
            matches: {
                Command::new("rinex-cli")
                    .author("Guillaume W. Bres, <guillaume.bressaix@gmail.com>")
                    .version(env!("CARGO_PKG_VERSION"))
                    .about("RINEX post processing")
                    .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .arg(Arg::new("filepath")
                        .short('f')
                        .long("fp")
                        .value_name("FILE")
                        .action(ArgAction::Append)
                        .required_unless_present("directory")
                        .help("Load a single file. Use this as many times as needed. 
Available operations and following behavior highly depends on input data. 
Supported formats are:
- Observation RINEX
- Navigation RINEX
- Meteo RINEX
- Clock RINEX (high precision clocks)
- SP3 (high precision orbits)
- IONEX (Ionosphere Maps)
- ANTEX (antenna calibration as RINEX)
- DORIS (special Observation RINEX)"))
                    .arg(Arg::new("directory")
                        .short('d')
                        .long("dir")
                        .value_name("DIRECTORY")
                        .action(ArgAction::Append)
                        .required_unless_present("filepath")
                        .help("Load directory recursively. Use this as many times as needed. Default recursive depth is set to 5,
but you can extend that with --depth. Refer to -f for more information."))
                    .arg(Arg::new("depth")
                        .long("depth")
                        .action(ArgAction::Set)
                        .required(false)
                        .value_parser(value_parser!(u8))
                        .help("Extend maximal recursive search depth of -d. The default is 5."))
                    .arg(Arg::new("quiet")
                        .short('q')
                        .long("quiet")
                        .action(ArgAction::SetTrue)
                        .help("Disable all terminal output. Also disables auto HTML reports opener."))
                    .arg(Arg::new("workspace")
                        .short('w')
                        .long("workspace")
                        .value_name("FOLDER")
                        .value_parser(value_parser!(PathBuf))
                        .help("Define custom workspace location. The env. variable RINEX_WORKSPACE, if present, is prefered.
If none of those exist, we will generate local \"WORKSPACE\" folder."))
        .next_help_heading("Preprocessing")
            .arg(Arg::new("gps-filter")
                .short('G')
                .action(ArgAction::SetTrue)
                .help("Filters out all GPS vehicles"))
            .arg(Arg::new("glo-filter")
                .short('R')
                .action(ArgAction::SetTrue)
                .help("Filters out all Glonass vehicles"))
            .arg(Arg::new("gal-filter")
                .short('E')
                .action(ArgAction::SetTrue)
                .help("Filters out all Galileo vehicles"))
            .arg(Arg::new("bds-filter")
                .short('C')
                .action(ArgAction::SetTrue)
                .help("Filters out all BeiDou vehicles"))
            .arg(Arg::new("qzss-filter")
                .short('J')
                .action(ArgAction::SetTrue)
                .help("Filters out all QZSS vehicles"))
            .arg(Arg::new("irnss-filter")
                .short('I')
                .action(ArgAction::SetTrue)
                .help("Filters out all IRNSS vehicles"))
            .arg(Arg::new("sbas-filter")
                .short('S')
                .action(ArgAction::SetTrue)
                .help("Filters out all SBAS vehicles"))
            .arg(Arg::new("preprocessing")
                .short('P')
                .num_args(1..)
                .action(ArgAction::Append)
                .help("Filter designer. Refer to []."))
            .arg(Arg::new("lli-mask")
                .long("lli-mask")
                .help("Applies given LLI AND() mask. 
Also drops observations that did not come with an LLI flag"))
            .next_help_heading("Receiver Antenna")
                .arg(Arg::new("rx-ecef")
                    .long("rx-ecef")
                    .value_name("\"x,y,z\" coordinates in ECEF [m]")
                    .help("Define the (RX) antenna position manually, in [m] ECEF.
Especially if your dataset does not define such position. 
Otherwise it gets automatically picked up."))
                .arg(Arg::new("rx-geo")
                    .long("rx-geo")
                    .value_name("\"lat,lon,alt\" coordinates in ddeg [°]")
                    .help("Define the (RX) antenna position manualy, in decimal degrees."))
                .next_help_heading("Exclusive Opmodes: you can only run one at a time.")
                .subcommand(filegen::subcommand())
                .subcommand(graph::subcommand())
                .subcommand(identify::subcommand())
                .subcommand(merge::subcommand())
                .subcommand(positioning::subcommand())
                .subcommand(qc::subcommand())
                .subcommand(split::subcommand())
                .subcommand(substract::subcommand())
                .subcommand(time_binning::subcommand())
                .get_matches()
            },
        }
    }
    /// Returns list of input directories
    pub fn input_directories(&self) -> Vec<&String> {
        if let Some(fp) = self.matches.get_many::<String>("directory") {
            fp.collect()
        } else {
            Vec::new()
        }
    }
    /// Returns individual input filepaths
    pub fn input_files(&self) -> Vec<&String> {
        if let Some(fp) = self.matches.get_many::<String>("filepath") {
            fp.collect()
        } else {
            Vec::new()
        }
    }
    pub fn preprocessing(&self) -> Vec<&String> {
        if let Some(filters) = self.matches.get_many::<String>("preprocessing") {
            filters.collect()
        } else {
            Vec::new()
        }
    }
    pub fn gps_filter(&self) -> bool {
        self.matches.get_flag("gps-filter")
    }
    pub fn glo_filter(&self) -> bool {
        self.matches.get_flag("glo-filter")
    }
    pub fn gal_filter(&self) -> bool {
        self.matches.get_flag("gal-filter")
    }
    pub fn bds_filter(&self) -> bool {
        self.matches.get_flag("bds-filter")
    }
    pub fn qzss_filter(&self) -> bool {
        self.matches.get_flag("qzss-filter")
    }
    pub fn sbas_filter(&self) -> bool {
        self.matches.get_flag("sbas-filter")
    }
    pub fn irnss_filter(&self) -> bool {
        self.matches.get_flag("irnss-filter")
    }
    /*
     * faillible 3D coordinates parsing
     * it's better to panic if the descriptor is badly format
     * then continuing with possible other coordinates than the
     * ones desired by user
     */
    fn parse_3d_coordinates(desc: &String) -> (f64, f64, f64) {
        let content = desc.split(',').collect::<Vec<&str>>();
        if content.len() < 3 {
            panic!("expecting x, y and z coordinates (3D)");
        }
        let x = f64::from_str(content[0].trim())
            .unwrap_or_else(|_| panic!("failed to parse x coordinates"));
        let y = f64::from_str(content[1].trim())
            .unwrap_or_else(|_| panic!("failed to parse y coordinates"));
        let z = f64::from_str(content[2].trim())
            .unwrap_or_else(|_| panic!("failed to parse z coordinates"));
        (x, y, z)
    }
    fn manual_ecef(&self) -> Option<(f64, f64, f64)> {
        let desc = self.matches.get_one::<String>("rx-ecef")?;
        let ecef = Self::parse_3d_coordinates(desc);
        Some(ecef)
    }
    fn manual_geodetic(&self) -> Option<(f64, f64, f64)> {
        let desc = self.matches.get_one::<String>("rx-geo")?;
        let geo = Self::parse_3d_coordinates(desc);
        Some(geo)
    }
    /// Returns RX Position possibly specified by user
    pub fn manual_position(&self) -> Option<(f64, f64, f64)> {
        if let Some(position) = self.manual_ecef() {
            Some(position)
        } else {
            self.manual_geodetic()
                .map(|position| GroundPosition::from_geodetic(position).to_ecef_wgs84())
        }
    }
}
