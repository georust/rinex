use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};
use gnss_rtk::prelude::Config;
use log::{error, info};
use rinex::prelude::*;
use rinex_qc::QcOpts;
use std::path::Path;
use std::str::FromStr;

pub struct Cli {
    /// Arguments passed by user
    matches: ArgMatches,
}

impl Cli {
    /// Build new command line interface
    pub fn new() -> Self {
        Self {
            matches: {
                Command::new("rinex-cli")
                    .author("Guillaume W. Bres, <guillaume.bressaix@gmail.com>")
                    .version(env!("CARGO_PKG_VERSION"))
                    .about("RINEX analysis and processing tool")
                    .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .arg(Arg::new("filepath")
                        .short('f')
                        .long("fp")
                        .value_name("FILE")
                        .action(ArgAction::Append)
                        .required_unless_present("directory")
                        .help("Input RINEX file. Can be any kind of RINEX, or an SP3 file,
and you can load as many as you want."))
                    .arg(Arg::new("directory")
                        .short('d')
                        .long("dir")
                        .value_name("DIRECTORY")
                        .action(ArgAction::Append)
                        .required_unless_present("filepath")
                        .help("Load directory recursively. RINEX and SP3 files are identified
and added like they were individually imported with -f.
You can import as many directories as you need."))
                    .arg(Arg::new("quiet")
                        .short('q')
                        .long("quiet")
                        .action(ArgAction::SetTrue)
                        .help("Disable all terminal output. Also disables auto HTML reports opener."))
                    .arg(Arg::new("pretty-json")
                        .short('j')
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("Make JSON output more readable."))
                    .arg(Arg::new("workspace")
                        .short('w')
                        .long("workspace")
                        .value_name("FOLDER")
                        .help("Customize workspace location (folder does not have to exist).
The default workspace is rinex-cli/workspace"))
                    .arg(Arg::new("no-graph")
                        .long("no-graph")
                        .action(ArgAction::SetTrue)
                        .help("Disable graphs generation, only text reports are to be generated."))
                .next_help_heading("Data generation")
					.arg(Arg::new("gpx")
						.long("gpx")
                        .action(ArgAction::SetTrue)
						.help("Enable GPX formatting. In RTK mode, a GPX track is generated."))
					.arg(Arg::new("kml")
						.long("kml")
                        .action(ArgAction::SetTrue)
						.help("Enable KML formatting. In RTK mode, a KML track is generated."))
                    .arg(Arg::new("output")
                        .short('o')
                        .long("out")
                        .value_name("FILE")
                        .action(ArgAction::Append)
                        .help("Custom file name to be generated within Workspace.
Allows merged file name to be customized."))
                .next_help_heading("Data identification")
                    .arg(Arg::new("full-id")
                        .short('i')
                        .action(ArgAction::SetTrue)
                        .help("Turn all identifications ON"))
                    .arg(Arg::new("epochs")
                        .long("epochs")
                        .action(ArgAction::SetTrue)
                        .help("Enumerate all epochs"))
                    .arg(Arg::new("gnss")
                        .long("gnss")
                        .short('g')
                        .action(ArgAction::SetTrue)
                        .help("Enumerate GNSS constellations present in entire context."))
                    .arg(Arg::new("sv")
                        .long("sv")
                        .action(ArgAction::SetTrue)
                        .help("Enumerate Sv"))
                    .arg(Arg::new("sampling")
                        .long("sampling")
                        .action(ArgAction::SetTrue)
                        .help("Sample rate analysis."))
                    .arg(Arg::new("header")
                        .long("header")
                        .action(ArgAction::SetTrue)
                        .help("Extracts major header fields"))
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
						.help("Design preprocessing operations, like data filtering or resampling,
prior further analysis. You can stack as many ops as you need.
Preprocessing ops apply prior entering both -q and --rtk modes.
Refer to rinex-cli/doc/preprocessing.md to learn how to operate this interface."))
                .next_help_heading("Observation RINEX")
                    .arg(Arg::new("observables")
                        .long("observables")
                        .long("obs")
                        .action(ArgAction::SetTrue)
                        .help("Identify observables in either Observation Data or Meteo Data contained in context."))
                    .arg(Arg::new("ssi-range")
                        .long("ssi-range")
                        .action(ArgAction::SetTrue)
                        .help("Display SSI (min,max) range, accross all epochs and vehicles"))
                    .arg(Arg::new("lli-mask")
                        .long("lli-mask")
                        .help("Applies given LLI AND() mask. 
Also drops observations that did not come with an LLI flag"))
                    .arg(Arg::new("if")
                        .long("if")
                        .action(ArgAction::SetTrue)
                        .help("Ionosphere Free combination graph"))
                    .arg(Arg::new("gf")
                        .long("gf")
                        .action(ArgAction::SetTrue)
                        .conflicts_with("no-graph")
                        .help("Geometry Free combination graph"))
                    .arg(Arg::new("wl")
                        .long("wl")
                        .action(ArgAction::SetTrue)
                        .conflicts_with("no-graph")
                        .help("Wide Lane combination graph"))
                    .arg(Arg::new("nl")
                        .long("nl")
                        .action(ArgAction::SetTrue)
                        .conflicts_with("no-graph")
                        .help("Narrow Lane combination graph"))
                    .arg(Arg::new("mw")
                        .long("mw")
                        .action(ArgAction::SetTrue)
                        .conflicts_with("no-graph")
                        .help("Melbourne-Wübbena combination graph"))
                    .arg(Arg::new("dcb")
                        .long("dcb")
                        .action(ArgAction::SetTrue)
                        .conflicts_with("no-graph")
                        .help("Differential Code Bias analysis"))
                    .arg(Arg::new("multipath")
                        .long("mp")
                        .action(ArgAction::SetTrue)
                        .conflicts_with("no-graph")
                        .help("Code Multipath analysis"))
                    .arg(Arg::new("anomalies")
                        .short('a')
                        .long("anomalies")
                        .action(ArgAction::SetTrue)
                        .help("Enumerate epochs where anomalies were reported by the receiver"))
                    .arg(Arg::new("cs")
                        .long("cs")
                        .action(ArgAction::SetTrue)
                        .help("Cycle Slip detection (graphical).
Helps visualize what the CS detector is doing and fine tune its operation.
CS do not get repaired with this command.
If you're just interested in CS information, you probably just want `-qc` instead, avoid combining the two."))
                    .arg(Arg::new("antenna-ecef")
                        .long("antenna-ecef")
                        .value_name("\"x,y,z\" coordinates in ECEF [m]")
                        .help("Define the (RX) antenna ground position manualy, in [m] ECEF system.
Some calculations require a reference position.
Ideally this information is contained in the file Header, but user can manually define them (superceeds)."))
                    .arg(Arg::new("antenna-lla")
                        .long("antenna-lla")
                        .value_name("\"lat,lon,alt\" coordinates in ddeg [°]")
                        .help("Define the (RX) antenna ground position manualy, in decimal degrees.
Some calculations require a reference position.
Ideally this information is contained in the file Header, but user can manually define them (superceeds)."))
                    .arg(Arg::new("nav-msg")
                        .long("nav-msg")
                        .action(ArgAction::SetTrue)
                        .help("Identify Navigation frame types. -fp must be a NAV file")) 
                    .arg(Arg::new("clock-bias")
                        .long("clock-bias")
                        .action(ArgAction::SetTrue)
                        .help("Display clock biases (offset, drift, drift changes) per epoch and vehicle.
-fp must be a NAV file"))
                .next_help_heading("Quality Check (QC)")
                    .arg(Arg::new("qc")
                        .long("qc")
                        .action(ArgAction::SetTrue)
                        .help("Enable Quality Check (QC) mode.
Runs thorough analysis on provided RINEX data.
The summary report by default is integrated to the global HTML report."))
					.arg(Arg::new("qc-config")
						.long("qc-cfg")
						.value_name("FILE")
						.help("Pass a QC configuration file."))
                    .arg(Arg::new("qc-only")
                        .long("qc-only")
                        .action(ArgAction::SetTrue)
                        .help("Activates QC mode and disables all other features: quickest qc rendition."))
                .next_help_heading("Positioning")
                    .arg(Arg::new("spp")
                        .long("spp")
                        .conflicts_with("ppp")
                        .action(ArgAction::SetTrue)
                        .help("Enable Single Point Positioning.
Use with ${RUST_LOG} env logger for more information.
Refer to the positioning documentation."))
                    .arg(Arg::new("ppp")
                        .long("ppp")
                        .conflicts_with("spp")
                        .action(ArgAction::SetTrue)
                        .help("Enable Precise Point Positioning.
Use with ${RUST_LOG} env logger for more information.
Refer to the positioning documentation."))
                    .arg(Arg::new("pos-only")
                        .long("pos-only")
                        .short('p')
                        .action(ArgAction::SetTrue)
                        .help("Disable context analysis and run position solver only.
This is the most performant mode to solve a position."))
					.arg(Arg::new("config")
						.long("cfg")
                        .short('c')
						.value_name("FILE")
						.help("Pass Positioning configuration, refer to doc/positioning."))
                .next_help_heading("File operations")
                    .arg(Arg::new("merge")
                        .short('m')
                        .long("merge")
                        .value_name("FILE(s)")
                        .action(ArgAction::Append)
                        .help("Merge given RINEX this RINEX into primary RINEX.
Primary RINEX was either loaded with `-f`, or is Observation RINEX loaded with `-d`"))
                    .arg(Arg::new("split")
                        .long("split")
                        .value_name("Epoch")
                        .short('s')
                        .help("Split RINEX into two separate files"))
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
    /// Returns output filepaths
    pub fn output_path(&self) -> Option<&String> {
        self.matches.get_one::<String>("output")
    }
    pub fn preprocessing(&self) -> Vec<&String> {
        if let Some(filters) = self.matches.get_many::<String>("preprocessing") {
            filters.collect()
        } else {
            Vec::new()
        }
    }
    pub fn quality_check(&self) -> bool {
        self.matches.get_flag("qc")
    }
    fn qc_config_path(&self) -> Option<&String> {
        if let Some(path) = self.matches.get_one::<String>("qc-config") {
            Some(path)
        } else {
            None
        }
    }
    pub fn qc_config(&self) -> QcOpts {
        if let Some(path) = self.qc_config_path() {
            if let Ok(content) = std::fs::read_to_string(path) {
                let opts = serde_json::from_str(&content);
                if let Ok(opts) = opts {
                    info!("qc parameter file \"{}\"", path);
                    opts
                } else {
                    error!("failed to parse parameter file \"{}\"", path);
                    info!("using default parameters");
                    QcOpts::default()
                }
            } else {
                error!("failed to read parameter file \"{}\"", path);
                info!("using default parameters");
                QcOpts::default()
            }
        } else {
            QcOpts::default()
        }
    }
    pub fn quality_check_only(&self) -> bool {
        self.matches.get_flag("qc-only")
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
    pub fn gf_combination(&self) -> bool {
        self.matches.get_flag("gf")
    }
    pub fn if_combination(&self) -> bool {
        self.matches.get_flag("if")
    }
    pub fn wl_combination(&self) -> bool {
        self.matches.get_flag("wl")
    }
    pub fn nl_combination(&self) -> bool {
        self.matches.get_flag("nl")
    }
    pub fn mw_combination(&self) -> bool {
        self.matches.get_flag("mw")
    }
    pub fn identification(&self) -> bool {
        self.matches.get_flag("sv")
            | self.matches.get_flag("epochs")
            | self.matches.get_flag("header")
            | self.matches.get_flag("observables")
            | self.matches.get_flag("ssi-range")
            | self.matches.get_flag("nav-msg")
            | self.matches.get_flag("anomalies")
            | self.matches.get_flag("sampling")
            | self.matches.get_flag("full-id")
    }
    /// Returns true if Sv accross epoch display is requested
    pub fn sv_epoch(&self) -> bool {
        self.matches.get_flag("sv-epoch")
    }
    /// Phase /PR DCBs analysis requested
    pub fn dcb(&self) -> bool {
        self.matches.get_flag("dcb")
    }
    /// Code Multipath analysis requested
    pub fn multipath(&self) -> bool {
        self.matches.get_flag("multipath")
    }
    /// Returns list of requested data to extract
    pub fn identification_ops(&self) -> Vec<&str> {
        if self.matches.get_flag("full-id") {
            vec![
                "sv",
                "epochs",
                "gnss",
                "observables",
                "ssi-range",
                "nav-msg",
                "anomalies",
                "sampling",
            ]
        } else {
            let flags = [
                "sv",
                "header",
                "sampling",
                "epochs",
                "gnss",
                "observables",
                "ssi-range",
                "nav-msg",
                "anomalies",
            ];
            flags
                .iter()
                .filter(|x| self.matches.get_flag(x))
                .copied()
                .collect()
        }
    }
    fn get_flag(&self, flag: &str) -> bool {
        self.matches.get_flag(flag)
    }
    /// returns true if pretty JSON is requested
    pub fn pretty_json(&self) -> bool {
        self.get_flag("pretty-json")
    }
    /// Returns true if quiet mode is activated
    pub fn quiet(&self) -> bool {
        self.matches.get_flag("quiet")
    }
    /// Returns true if SPP position solver is enabled
    pub fn spp(&self) -> bool {
        self.matches.get_flag("spp")
    }
    pub fn ppp(&self) -> bool {
        self.matches.get_flag("spp")
    }
    pub fn positioning_only(&self) -> bool {
        self.matches.get_flag("pos-only")
    }
    pub fn gpx(&self) -> bool {
        self.matches.get_flag("gpx")
    }
    pub fn kml(&self) -> bool {
        self.matches.get_flag("kml")
    }
    pub fn config(&self) -> Option<Config> {
        if let Some(path) = self.matches.get_one::<String>("config") {
            if let Ok(content) = std::fs::read_to_string(path) {
                let opts = serde_json::from_str(&content);
                if let Ok(opts) = opts {
                    info!("loaded rtk config: \"{}\"", path);
                    return Some(opts);
                } else {
                    panic!("failed to parse config file \"{}\"", path);
                }
            } else {
                error!("failed to read config file \"{}\"", path);
                info!("using default parameters");
            }
        }
        None
    }
    pub fn cs_graph(&self) -> bool {
        self.matches.get_flag("cs")
    }
    /*
     * No graph to be generated
     */
    pub fn no_graph(&self) -> bool {
        self.matches.get_flag("no-graph")
    }
    /*
     * Returns possible file path to merge
     */
    pub fn merge_path(&self) -> Option<&Path> {
        self.matches.get_one::<String>("merge").map(Path::new)
    }
    /// Returns optionnal RINEX file to "merge"
    pub fn to_merge(&self) -> Option<Rinex> {
        if let Some(path) = self.merge_path() {
            let path = path.to_str().unwrap();
            match Rinex::from_file(path) {
                Ok(rnx) => Some(rnx),
                Err(e) => {
                    error!("failed to parse \"{}\", {}", path, e);
                    None
                },
            }
        } else {
            None
        }
    }
    /// Returns split operation args
    pub fn split(&self) -> Option<Epoch> {
        if self.matches.contains_id("split") {
            if let Some(s) = self.matches.get_one::<String>("split") {
                if let Ok(epoch) = Epoch::from_str(s) {
                    Some(epoch)
                } else {
                    panic!("failed to parse [EPOCH]");
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    fn manual_ecef(&self) -> Option<&String> {
        self.matches.get_one::<String>("antenna-ecef")
    }
    fn manual_geodetic(&self) -> Option<&String> {
        self.matches.get_one::<String>("antenna-geo")
    }
    /// Returns Ground Position possibly specified by user
    pub fn manual_position(&self) -> Option<GroundPosition> {
        if let Some(args) = self.manual_ecef() {
            let content: Vec<&str> = args.split(',').collect();
            if content.len() != 3 {
                panic!("expecting \"x, y, z\" description");
            }
            if let Ok(pos_x) = f64::from_str(content[0].trim()) {
                if let Ok(pos_y) = f64::from_str(content[1].trim()) {
                    if let Ok(pos_z) = f64::from_str(content[2].trim()) {
                        return Some(GroundPosition::from_ecef_wgs84((pos_x, pos_y, pos_z)));
                    } else {
                        error!("pos(z) should be f64 ECEF [m]");
                    }
                } else {
                    error!("pos(y) should be f64 ECEF [m]");
                }
            } else {
                error!("pos(x) should be f64 ECEF [m]");
            }
        } else if let Some(args) = self.manual_geodetic() {
            let content: Vec<&str> = args.split(',').collect();
            if content.len() != 3 {
                panic!("expecting \"lat, lon, alt\" description");
            }
            if let Ok(lat) = f64::from_str(content[0].trim()) {
                if let Ok(long) = f64::from_str(content[1].trim()) {
                    if let Ok(alt) = f64::from_str(content[2].trim()) {
                        return Some(GroundPosition::from_geodetic((lat, long, alt)));
                    } else {
                        error!("altitude should be f64 [ddeg]");
                    }
                } else {
                    error!("altitude should be f64 [ddeg]");
                }
            } else {
                error!("altitude should be f64 [ddeg]");
            }
        }
        None
    }
}
