use std::{
    collections::hash_map::DefaultHasher,
    fs::File,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    str::FromStr,
};

use itertools::Itertools;

use clap::{value_parser, Arg, ArgAction, ArgMatches, ColorChoice, Command};
use rinex_qc::{
    cfg::{
        QcCustomRoverOpts, QcNaviOpts, QcPreferedClock, QcPreferedOrbit, QcPreferedRover,
        QcPreferedRoversSorting, QcPreferedSettings, QcReportOpts, QcReportType, QcSolutions,
    },
    prelude::{QcConfig, QcContext},
};

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
                    .about("RINEX and SP3 post processing")
                    .long_about("rinex-cli is a command line tool to post process
RINEX and SP3 data. It is based on the combination
of the RINE, SP3, RTK and Nyx libraries.")
                    .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .next_help_heading("Context (Data stacking & definitions)")
                    .arg(Arg::new("filepath")
                        .long("fp")
                        .short('f')
                        .value_name("FILE")
                        .action(ArgAction::Append)
                        .required_unless_present("directory")
                        .help("Load a single file. At least one file is always expected. See --help for more information.")
                        .long_help("Use --fp,-f for as many individual files you may need (whatever their kind).
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

Example (2): define a PPP compliant context by stacking several files
rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -f test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \\
    -f test_resources/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \\ 
    -f test_resources/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz
"))
                    .arg(Arg::new("directory")
                        .short('d')
                        .long("dir")
                        .value_name("DIRECTORY")
                        .action(ArgAction::Append)
                        .required_unless_present("filepath")
                        .help("Use --dir,-d to stack all files contained in given directory. See --help for more information.")
                        .long_help("Use --dir,-d as many times as you need, the total amount of files being unlimited. 
The default recursive depth we search for is 5.
Use --depth to increase this value.
This makes the toolbox compliants with any folder organization.
For example FORMAT/YEAR/DOY/Hours."))
                    .arg(Arg::new("depth")
                        .long("depth")
                        .action(ArgAction::Set)
                        .required(false)
                        .help("Custom --dir,-d maximal recursive depth")
                        .value_parser(value_parser!(u8)))
                    .next_help_heading("Session (custom preferences)")
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
                        .help("Define custom workspace location. See --help for more information.")
                        .long_help("Workspace is where Output Products are to be generated.
Whether they are text files or HTML reports.
The $GEORUST_WORKSPACE variable is automatically picked up by this application and always prefered.
Use --workspace,-w to define it at runtime if you prefer.
When no workspace is defined, we simply a create local folder."))
                    .arg(Arg::new("zip")
                        .long("zip")
                        .action(ArgAction::SetTrue)
                        .help("Gzip compress any output directly.
This may apply to both text files and HTML reports!"))
        .next_help_heading("Preprocessor (Filter Designer)")
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

    // Parse 3d coordinates from command line
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

    fn manual_rx_ecef(&self) -> Option<(f64, f64, f64)> {
        let desc = self.matches.get_one::<String>("rx-ecef")?;
        let ecef = Self::parse_3d_coordinates(desc);
        Some(ecef)
    }

    fn manual_rx_geodetic(&self) -> Option<(f64, f64, f64)> {
        let desc = self.matches.get_one::<String>("rx-geo")?;
        let geo = Self::parse_3d_coordinates(desc);
        Some(geo)
    }

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

    /// Returns QcConfig from command line
    pub fn qc_config(&self) -> QcConfig {
        QcConfig::default()
            .with_workspace(&self.workspace_path())
            .with_preferences(self.prefered_settings())
            .with_rover_settings(self.rover_settings())
            .with_report_preferences(self.report_preferences())
            .with_solutions(self.solutions())
    }

    fn prefered_settings(&self) -> QcPreferedSettings {
        QcPreferedSettings {
            clk_source: QcPreferedClock::RINEx,
            orbit_source: QcPreferedOrbit::RINEx,
            rovers_sorting: QcPreferedRoversSorting::Receiver,
        }
    }

    fn solutions(&self) -> QcSolutions {
        QcSolutions {
            ppp: true,
            cggtts: true,
        }
    }

    fn report_preferences(&self) -> QcReportOpts {
        QcReportOpts {
            report_type: if self.matches.get_flag("summary") {
                QcReportType::Summary
            } else {
                QcReportType::Full
            },
            signal_combinations: false,
        }
    }

    fn rover_settings(&self) -> QcCustomRoverOpts {
        QcCustomRoverOpts {
            manual_rx_ecef_km: if let Some((x_ecef_km, y_ecef_km, z_ecef_km)) =
                self.manual_rx_ecef()
            {
                Some((x_ecef_km, y_ecef_km, z_ecef_km))
            } else if let Some((lat_ddeg, long_ddeg, alt_km)) = self.manual_rx_geodetic() {
                panic!("not supported yet. Use ECEF specification");
            } else {
                None
            },
            prefered_rover: self.prefered_rover(),
        }
    }

    fn prefered_rover(&self) -> QcPreferedRover {
        QcPreferedRover::Any
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
