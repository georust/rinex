use std::{
    collections::hash_map::DefaultHasher,
    fs::File,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    str::FromStr,
};

use itertools::Itertools;

use clap::{value_parser, Arg, ArgAction, ArgMatches, ColorChoice, Command};
use rinex_qc::prelude::{QcConfig, QcContext, QcReportType};

mod fops;
mod positioning;

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
pub struct CliContext {
    /// Quiet option
    pub quiet: bool,
    /// Data context defined by user, expressed as [QcContext].
    pub qc_context: QcContext,
}

impl Cli {
    /// Build new command line interface
    pub fn new() -> Self {
        let cmd =
                Command::new("rinex-cli")
                    .author("Guillaume W. Bres <guillaume.bressaix@gmail.com>")
                    .version(env!("CARGO_PKG_VERSION"))
                    .about("RINex and SP3 post processing")
                    .long_about("RINex-cli is a command line
to post process RINex and possibly stacked SP3 data.
It uses the RINex, SP3, RTK and Nyx-Space ecosystem.")
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
By default the $GEORUST_WORKSPACE variable is prefered.
Use -w to manually define it and avoid using the environment variable.
When no workspace is defined, we simply create a local folder."))
                    .arg(Arg::new("zip")
                        .long("zip")
                        .action(ArgAction::SetTrue)
                        .help("Gzip compress your output directly.
NB: this applies to all output product, whether they are data files like RINex or HTML reports."))
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
        .next_help_heading("Data Reporting")
        .arg(
            Arg::new("summary")
                .long("sum")
                .action(ArgAction::SetTrue)
                .help("Restrict report to a summary only (quicker rendition)")
        )
        .arg(
            Arg::new("output-name")
                .short('o')
                .action(ArgAction::Set)
                .help("Customize output file or report name.
In analysis opmode, report is named index.html by default, this will redefine that.
In file operations (filegen, etc..) we can manually define output filenames with this option."))
        .arg(
            Arg::new("report-force")
                .short('f')
                .long("force")
                .action(ArgAction::SetTrue)
                .help("Force report synthesis.
By default, report synthesis happens once per input set (file combnation and cli options).
Use this option to force report regeneration.
This has no effect on file operations that do not synthesize a report."))
            .next_help_heading("RINEX Data (specific)")
                .arg(
                    Arg::new("rnx2crx")
                        .long("rnx2crx")
                        .action(ArgAction::SetTrue)
                        .help("Convert all observations to CRINEX (if any).
In file synthesis, we will only generate compressed RINEX.")
                )
                .arg(
                    Arg::new("crx2rnx")
                        .long("crx2rnx")
                        .action(ArgAction::SetTrue)
                        .help("Convert all compressed observations to RINEX (if any).
In file synthesis, we will only generate readable RINEX.")
                )
                .arg(Arg::new("zero-repair")
                    .short('z')
                    .action(ArgAction::SetTrue)
                    .help("(Zero Repair) remove all zero (=null) values. See --help")
                    .long_help("
Removes all zero (null) values from data records.
NB: this does not apply every possible fields but specific.
When applied to NAV RINEx: Null NAV records are always forbidden so we repair them.
When applied to OBS RINEx: we null Phase and Decoded Ranges are patched, because they are physically
incorrect and most likely, the result of a receiver internal error.
Use the geodetic report to monitor whether your data contains incorrect data points.
The `ppp` mode will report physical non-sense on such data points."))
                .arg(Arg::new("short-rinex")
                    .short('s')
                    .long("short")
                    .action(ArgAction::SetTrue)
                    .help("Prefer (V2) short file names when synthesizing RINex files.
NB: this toolbox uses V3 / long file names by default.
NB(2): this only applies to Meteo, Observation and Navigation RINex."))
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
                .subcommand(filegen::subcommand());

        let cmd = cmd
            .subcommand(merge::subcommand())
            .subcommand(positioning::ppp_subcommand())
            .subcommand(positioning::rtk_subcommand())
            .subcommand(split::subcommand())
            .subcommand(diff::subcommand())
            .subcommand(time_binning::subcommand());
        Self {
            matches: cmd.get_matches(),
        }
    }

    /// Recursive browser depth
    pub fn recursive_depth(&self) -> usize {
        if let Some(depth) = self.matches.get_one::<u8>("depth") {
            *depth as usize
        } else {
            5
        }
    }

    /// True when -q (quiet) option is active
    pub fn quiet(&self) -> bool {
        self.matches.get_flag("quiet")
    }

    /// True if forced report synthesis is requested
    pub fn force_report_synthesis(&self) -> bool {
        self.matches.get_flag("report-force")
    }

    /// Returns individual input ROVER -d
    pub fn rover_directories(&self) -> Vec<&String> {
        if let Some(dirs) = self.matches.get_many::<String>("directory") {
            dirs.collect()
        } else {
            Vec::new()
        }
    }

    /// Returns individual input ROVER -fp
    pub fn rover_files(&self) -> Vec<&String> {
        if let Some(fp) = self.matches.get_many::<String>("filepath") {
            fp.collect()
        } else {
            Vec::new()
        }
    }

    /// Returns individual input BASE STATION -d
    pub fn base_station_directories(&self) -> Vec<&String> {
        match self.matches.subcommand() {
            Some(("rtk", submatches)) => {
                if let Some(dir) = submatches.get_many::<String>("dir") {
                    dir.collect()
                } else {
                    Vec::new()
                }
            },
            _ => Vec::new(),
        }
    }

    /// Returns individual input BASE STATION -fp
    pub fn base_station_files(&self) -> Vec<&String> {
        match self.matches.subcommand() {
            Some(("rtk", submatches)) => {
                if let Some(fp) = submatches.get_many::<String>("fp") {
                    fp.collect()
                } else {
                    Vec::new()
                }
            },
            _ => Vec::new(),
        }
    }

    /// Returns preproc ops
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

    pub fn zero_repair(&self) -> bool {
        self.matches.get_flag("zero-repair")
    }

    pub fn rnx2crx(&self) -> bool {
        self.matches.get_flag("rnx2crx")
    }

    pub fn crx2rnx(&self) -> bool {
        self.matches.get_flag("crx2rnx")
    }

    // /*
    //  * faillible 3D coordinates parsing
    //  * it's better to panic if the descriptor is badly format
    //  * then continuing with possible other coordinates than the
    //  * ones desired by user
    //  */
    // fn parse_3d_coordinates(desc: &String) -> (f64, f64, f64) {
    //     let content = desc.split(',').collect::<Vec<&str>>();
    //     if content.len() < 3 {
    //         panic!("expecting x, y and z coordinates (3D)");
    //     }
    //     let x = f64::from_str(content[0].trim())
    //         .unwrap_or_else(|_| panic!("failed to parse x coordinates"));
    //     let y = f64::from_str(content[1].trim())
    //         .unwrap_or_else(|_| panic!("failed to parse y coordinates"));
    //     let z = f64::from_str(content[2].trim())
    //         .unwrap_or_else(|_| panic!("failed to parse z coordinates"));
    //     (x, y, z)
    // }

    // fn manual_ecef(&self) -> Option<(f64, f64, f64)> {
    //     let desc = self.matches.get_one::<String>("rx-ecef")?;
    //     let ecef = Self::parse_3d_coordinates(desc);
    //     Some(ecef)
    // }

    // fn manual_geodetic(&self) -> Option<(f64, f64, f64)> {
    //     let desc = self.matches.get_one::<String>("rx-geo")?;
    //     let geo = Self::parse_3d_coordinates(desc);
    //     Some(geo)
    // }

    // /// Returns RX Position possibly specified by user
    // pub fn manual_position(&self) -> Option<(f64, f64, f64)> {
    //     if let Some(position) = self.manual_ecef() {
    //         Some(position)
    //     } else {
    //         self.manual_geodetic()
    //             .map(|position| GroundPosition::from_geodetic(position).to_ecef_wgs84())
    //     }
    // }

    /// True if File Operations to generate data is being deployed
    pub fn has_fops_output_product(&self) -> bool {
        matches!(
            self.matches.subcommand(),
            Some(("filegen", _))
                | Some(("merge", _))
                | Some(("split", _))
                | Some(("tbin", _))
                | Some(("diff", _))
        )
    }

    /*
     * We hash all vital CLI information.
     * This helps in determining whether we need to update an existing report
     * or not.
     */
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        let mut string = self
            .rover_directories()
            .into_iter()
            .sorted()
            .chain(self.rover_files().into_iter().sorted())
            .chain(self.preprocessing().into_iter().sorted())
            .join(",");
        if let Some(custom) = self.custom_output_name() {
            string.push_str(custom);
        }
        // if let Some(geo) = self.manual_geodetic() {
        //     string.push_str(&format!("{:?}", geo));
        // }
        // if let Some(ecef) = self.manual_ecef() {
        //     string.push_str(&format!("{:?}", ecef));
        // }
        string.hash(&mut hasher);
        hasher.finish()
    }

    /// Returns QcConfig from command line
    pub fn qc_config(&self) -> QcConfig {
        QcConfig::default().with_workspace(&self.workspace_path())
    }

    fn workspace_path(&self) -> String {
        if let Ok(workspace) = std::env::var("GEORUST_WORKSPACE") {
            workspace.to_string()
        } else {
            if let Some(workspace) = self.matches.get_one::<String>("workspace") {
                workspace.to_string()
            } else {
                "WORKSPACE".to_string()
            }
        }
    }

    /// Customized / manually defined output to be generated
    pub fn custom_output_name(&self) -> Option<&String> {
        self.matches.get_one::<String>("output-name")
    }

    /// Prefer short V2 like RINex file names
    pub fn short_rinex_file_name(&self) -> bool {
        self.matches.get_flag("short-rinex")
    }

    /// Apply gzip compression out the output
    pub fn gzip_encoding(&self) -> bool {
        self.matches.get_flag("zip")
    }
}
