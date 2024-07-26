use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    str::FromStr,
};

use itertools::Itertools;

use clap::{value_parser, Arg, ArgAction, ArgMatches, ColorChoice, Command};
use rinex::prelude::GroundPosition;
use rinex_qc::prelude::{QcConfig, QcContext, QcReportType};

mod fops;
mod positioning;
mod workspace;

pub use workspace::Workspace;

use fops::{diff, filegen, merge, split, time_binning};

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
    /// Quiet option
    pub quiet: bool,
    /// Data context defined by user
    pub data: QcContext,
    /// Context name is derived from the primary file loaded in Self,
    /// and mostly used in output products generation.
    pub name: String,
    /// Workspace is the place where this session will generate data.
    /// By default it is set to $WORKSPACE/$PRIMARYFILE.
    /// $WORKSPACE is either manually definedd by CLI or we create it (as is).
    /// $PRIMARYFILE is determined from the most major file contained in the dataset.
    pub workspace: Workspace,
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
    pub fn context_stem(data: &QcContext) -> String {
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
     * Utility to create a file in this session
     */
    fn create_file(&self, path: &Path) -> std::fs::File {
        std::fs::File::create(path).unwrap_or_else(|e| {
            panic!("failed to create {}: {:?}", path.display(), e);
        })
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
                    .long_about("RINEX-Cli is the command line interface
to operate the RINEX/SP3/RTK toolkit, until a GUI is made available.
Use it to analyze data, perform file operations and resolve navigation solutions.")
                    .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .arg(Arg::new("filepath")
                        .long("fp")
                        .value_name("FILE")
                        .action(ArgAction::Append)
                        .required_unless_present("directory")
                        .help("Load a single file. See --help")
                        .long_help("Use this as many times as needed. 
Available operations and following behavior highly depends on input data. 
Supported formats are:
- Observation RINEX
- Navigation RINEX
- Meteo RINEX
- Clock RINEX (high precision clocks)
- SP3 (high precision orbits)
- IONEX (Ionosphere Maps)
- ANTEX (antenna calibration as RINEX)
- DORIS (special Observation RINEX)

Example (1): Load a single file
rinex-cli \\
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz

Example (2): define a PPP compliant context
rinex-cli \\
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \\
    --fp test_resources/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \\ 
    --fp test_resources/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz
"))
                    .arg(Arg::new("directory")
                        .short('d')
                        .long("dir")
                        .value_name("DIRECTORY")
                        .action(ArgAction::Append)
                        .required_unless_present("filepath")
                        .help("Directory recursivel loader. See --help.")
                        .long_help("Use this as many times as needed. Default recursive depth is set to 5,
but you can extend that with --depth. Refer to -f for more information."))
                    .arg(Arg::new("depth")
                        .long("depth")
                        .action(ArgAction::Set)
                        .required(false)
                        .value_parser(value_parser!(u8))
                        .help("Extend maximal recursive search depth of -d. The default is 5.")
                        .long_help("The default recursive depth already supports hierarchies like:
/YEAR1
     /DOY0
          /STATION1
     /DOY1
          /STATION2
/YEAR2
     /DOY0
          /STATION1"))
                    .arg(Arg::new("quiet")
                        .short('q')
                        .long("quiet")
                        .action(ArgAction::SetTrue)
                        .help("Disable all terminal output. Disables automatic report opener (Web browser)."))
                    .arg(Arg::new("workspace")
                        .short('w')
                        .long("workspace")
                        .value_name("FOLDER")
                        .value_parser(value_parser!(PathBuf))
                        .help("Define custom workspace location. See --help.")
                        .long_help("The Workspace is where Output Products are to be generated.
By default the $RINEX_WORKSPACE variable is prefered if it is defined.
You can also use this flag to customize it. 
If none are defined, we will then try to create a local directory named \"WORKSPACE\" like it is possible in this very repo."))
        .next_help_heading("Report customization")
        .arg(
            Arg::new("report-sum")
                .long("sum")
                .action(ArgAction::SetTrue)
                .help("Restrict report to summary header only (quicker rendition)")
        )
        .arg(
            Arg::new("report-force")
                .short('f')
                .long("force")
                .action(ArgAction::SetTrue)
                .help("Force report synthesis.
By default, report synthesis happens once per input set (file combnation and cli options).
Use this option to force report regeneration.
This has no effect on file operations that do not synthesize a report."))
        .arg(
            Arg::new("report-brdc-sky")
                .long("brdc-sky")
                .action(ArgAction::SetTrue)
                .help("When SP3 and/or BRDC RINEX is present,
the skyplot (compass) projection is only calculated from the SP3 coordinates (highest precision). Use this option to also calculate it from radio messages (for comparison purposes for example).")
        )
        .arg(
            Arg::new("report-nostats")
                .long("nostats")
                .action(ArgAction::SetTrue)
                .help("Hide statistical annotations that might be present in some plots.
This has no effect on applications compiled without plot and statistical options.")
        )
        .next_help_heading("Preprocessing")
            .about("Preprocessing todo")
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
            .arg(Arg::new("bds-geo-filter")
                .long("CG")
                .action(ArgAction::SetTrue)
                .help("Filter out all BeiDou Geo vehicles"))
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
                .value_delimiter(';')
                .action(ArgAction::Append)
                .help("Filter designer. Refer to []."))
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
                .subcommand(merge::subcommand())
                .subcommand(positioning::subcommand())
                .subcommand(split::subcommand())
                .subcommand(diff::subcommand())
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
    pub fn bds_geo_filter(&self) -> bool {
        self.matches.get_flag("bds-geo-filter")
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
    /// True if File Operations to generate data is being deployed
    pub fn has_fops_output_product(&self) -> bool {
        match self.matches.subcommand() {
            Some(("filegen", _)) | Some(("merge", _)) | Some(("split", _)) | Some(("tbin", _))
            | Some(("diff", _)) => true,
            _ => false,
        }
    }
    /// True if forced report synthesis is requested
    pub fn force_report_synthesis(&self) -> bool {
        self.matches.get_flag("report-force")
    }
    /*
     * We hash all vital CLI information.
     * This helps in determining whether we need to update an existing report
     * or not.
     */
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        let mut string = self
            .input_directories()
            .into_iter()
            .sorted()
            .chain(self.input_files().into_iter().sorted())
            .chain(self.preprocessing().into_iter().sorted())
            .join(",");
        if let Some(geo) = self.manual_geodetic() {
            string.push_str(&format!("{:?}", geo));
        }
        if let Some(ecef) = self.manual_ecef() {
            string.push_str(&format!("{:?}", ecef));
        }
        string.hash(&mut hasher);
        hasher.finish()
    }
    /// Returns QcConfig from command line
    pub fn qc_config(&self) -> QcConfig {
        QcConfig {
            manual_reference: None,
            report: if self.matches.get_flag("report-sum") {
                QcReportType::Summary
            } else {
                QcReportType::Full
            },
            force_brdc_skyplot: self.matches.get_flag("report-brdc-sky"),
        }
    }
}
