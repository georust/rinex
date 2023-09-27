use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};
use log::{error, info};
use rinex::prelude::*;
use rinex_qc::QcOpts;
use std::fs::ReadDir;
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
                        .help("Input RINEX file. Serves as primary data.
Must be Observation Data for --rtk.
Observation, Meteo and IONEX can only serve as primary data.")
                        .action(ArgAction::Append)
                        .required(true))
                .next_help_heading("General")
                    .arg(Arg::new("quiet")
                        .short('q')
                        .long("quiet")
                        .action(ArgAction::SetTrue)
                        .help("Disable all terminal output. Also disables auto HTML reports opener."))
                    .arg(Arg::new("pretty")
                        .short('p')
                        .long("pretty")
                        .action(ArgAction::SetTrue)
                        .help("Make terminal output more readable."))
                    .arg(Arg::new("workspace")
                        .short('w')
                        .long("workspace")
                        .value_name("FOLDER")
                        .help("Customize workspace location (folder does not have to exist).
The default workspace is rinex-cli/workspace"))
                .next_help_heading("Data identification")
                    .arg(Arg::new("epochs")
                        .long("epochs")
                        .action(ArgAction::SetTrue)
                        .help("Enumerate all epochs"))
                    .arg(Arg::new("constellations")
                        .long("constellations")
                        .short('c')
                        .action(ArgAction::SetTrue)
                        .help("Enumerate GNSS constellations"))
                    .arg(Arg::new("sv")
                        .long("sv")
                        .action(ArgAction::SetTrue)
                        .help("Enumerate Sv"))
                    .arg(Arg::new("sv-epoch")
                        .long("sv-epoch")
                        .action(ArgAction::SetTrue)
                        .help("Plot SV against Epoch.
Useful to determine common Epochs or compare sample rates in between 
--fp OBS and --nav NAV for example."))
                    .arg(Arg::new("sampling-hist")
                        .long("sampling-hist")
                        .action(ArgAction::SetTrue)
                        .help("Sample rate histogram analysis."))
                    .arg(Arg::new("header")
                        .long("header")
                        .action(ArgAction::SetTrue)
                        .help("Extracts (all) header fields"))
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
						.help("Design preprocessing operations, like data filtering or resampling,
prior further analysis. You can stack as many ops as you need.
Preprocessing ops apply prior entering both -q and --rtk modes.
Refer to rinex-cli/doc/preprocessing.md to learn how to operate this interface."))
                .next_help_heading("Observation RINEX")
                    .arg(Arg::new("observables")
                        .long("observables")
                        .short('o')
                        .action(ArgAction::SetTrue)
                        .help("Identify observables. Applies to Observation and Meteo RINEX"))
                    .arg(Arg::new("ssi-range")
                        .long("ssi-range")
                        .action(ArgAction::SetTrue)
                        .help("Display SSI (min,max) range, accross all epochs and vehicles"))
                    .arg(Arg::new("ssi-sv-range")
                        .long("ssi-sv-range")
                        .action(ArgAction::SetTrue)
                        .help("Extract SSI (min,max) range, per vehicle, accross all epochs"))
                    .arg(Arg::new("lli-mask")
                        .long("lli-mask")
                        .help("Applies given LLI AND() mask. 
Also drops observations that did not come with an LLI flag"))
                    .arg(Arg::new("clock-offset")
                        .long("clk")
                        .action(ArgAction::SetTrue)
                        .help("Receiver Clock offset / drift analysis."))
                    .arg(Arg::new("phase")
                        .long("phase")
                        .action(ArgAction::SetTrue)
                        .help("Plot phase data as is (do not converted to carrier cycles, still set phase(t=0)=0"))
                    .arg(Arg::new("raw-phase")
                        .long("raw-phase")
                        .action(ArgAction::SetTrue)
                        .help("Plot phase data as is (not aligned to origin, not converted to carrier cycles)"))
                    .arg(Arg::new("gf")
                        .long("gf")
                        .action(ArgAction::SetTrue)
                        .help("Geometry Free recombination of both Phase and PR measurements."))
                    .arg(Arg::new("wl")
                        .long("wl")
                        .action(ArgAction::SetTrue)
                        .help("Wide Lane recombination of both Phase and PR measurements."))
                    .arg(Arg::new("nl")
                        .long("nl")
                        .action(ArgAction::SetTrue)
                        .help("Narrow Lane recombination of both Phase and PR measurements."))
                    .arg(Arg::new("mw")
                        .long("mw")
                        .action(ArgAction::SetTrue)
                        .help("Melbourne-Wübbena recombinations"))
                    .arg(Arg::new("dcb")
                        .long("dcb")
                        .action(ArgAction::SetTrue)
                        .help("Differential Code Bias analysis"))
                    .arg(Arg::new("multipath")
                        .long("mp")
                        .action(ArgAction::SetTrue)
                        .help("Code Multipath analysis"))
                    .arg(Arg::new("iono")
                        .long("iono")
                        .action(ArgAction::SetTrue)
                        .help("Plot the ionospheric delay detector"))
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
                .next_help_heading("Navigation RINEX")
                    .arg(Arg::new("nav")
                        .long("nav")
                        .num_args(1..)
                        .value_name("FILE/FOLDER")
                        .action(ArgAction::Append)
                        .help("Local NAV RINEX file(s). Enhance given context with Navigation Data.
Use this flag to either load directories containing your Navigation data, 
or once per individual files. You can stack as many as you want.
Most useful when combined to Observation RINEX.  
Enables complete `--qc` analysis with elevation mask taken into account.")) 
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
                    .arg(Arg::new("orbits")
                        .long("orbits")
                        .action(ArgAction::SetTrue)
                        .help("Enumerate orbit fields."))
                    .arg(Arg::new("nav-msg")
                        .long("nav-msg")
                        .action(ArgAction::SetTrue)
                        .help("Identify Navigation frame types. -fp must be a NAV file")) 
                    .arg(Arg::new("clock-bias")
                        .long("clock-bias")
                        .action(ArgAction::SetTrue)
                        .help("Display clock biases (offset, drift, drift changes) per epoch and vehicle.
-fp must be a NAV file"))
                .next_help_heading("High Precision Orbit / Clock")
                    .arg(Arg::new("sp3")
                        .long("sp3")
                        .num_args(1..)
                        .value_name("FILE/FOLDER")
                        .action(clap::ArgAction::Append)
                        .help("Local SP3 file(s). Enhance given context with IGS high precision Orbits.
Use this flag to either load directories containing your SP3 data,
or once per individual files. You can stack as many as you want. 
Combining --sp3 and --nav unlocks residual comparison between the two datasets."))
                .next_help_heading("Antenna")
                    .arg(Arg::new("atx")
                        .long("atx")
						.num_args(1..)
                        .value_name("FILE/FOLDER")
                        .action(ArgAction::Append)
                        .help("Local ANTEX file(s). Enhance given context with ANTEX Data.
Use this flag to either load directories containing your ATX data,
or once per individual files. You can stack as many as you want."))
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
                .next_help_heading("RTK (Positioning)")
                    .arg(Arg::new("rtk")
                        .short('r')
                        .long("rtk")
                        .action(ArgAction::SetTrue)
                        .help("Activate GNSS receiver position solver.
This is only possible if provided context is sufficient.
Depending on provided context, either SPP (high accuracy) or PPP (ultra high accuracy)
solver is deployed.
This mode is turned off by default because it involves quite heavy computations.
Use the RUST_LOG env. variable for verbosity.
See [spp] for more information. "))
                    .arg(Arg::new("spp")
                        .long("spp")
                        .action(ArgAction::SetTrue)
                        .help("Enables Positioning forced to Single Frequency SPP solver mode.
Disregards whether the provided context is PPP compatible. 
NB: we do not account for Relativistic effects by default and raw pseudo range are used.
For indepth customization, refer to the configuration file and online documentation."))
                    .arg(Arg::new("rtk-only")
                        .long("rtk-only")
                        .action(ArgAction::SetTrue)
                        .help("Activates GNSS position solver, disables all other modes.
This is the most performant mode to solve a position."))
                    .arg(Arg::new("rtk-fixed-altitude")
                        .long("rtk-fixed-alt")
                        .value_name("ALTITUDE(f64)")
                        .help("Set rtk solver to fixed altitude mode.
Problem is simplified by not caring about the Z axis resolution."))
                    .arg(Arg::new("rtk-static")
                        .long("rtk-static")
                        .help("Set rtk solver to static mode.
Problem is simplified but will not work in case the receiver is not maintained in static position.
Works well in laboratory conditions.
Combine --rtk-fixed-alt --rtk-static is most efficient solving scenario."))
                    .arg(Arg::new("rtk-model")
                        .long("rtk-model")
                        .action(ArgAction::Append)
                        .help("Stack one modelization to account for when solving.
--model=tgd : account for SV total group delay
--model=smoothing: smooth pseudo ranges. This is pointless if you requested
the hatch filter with -P.
--model=eclipse:f64 : adjust minimal light rate to determine eclipse condition.
--model=tgd : account for SV total group delay")) 
                    .arg(Arg::new("kml")
                        .long("kml")
                        .help("Form a KML track with resolved positions.
This turns off the default visualization."))
                .next_help_heading("File operations")
                    .arg(Arg::new("merge")
                        .short('m')
                        .value_name("FILE")
                        .long("merge")
                        .help("Merges this RINEX into `--fp`"))
                    .arg(Arg::new("split")
                        .long("split")
                        .value_name("Epoch")
                        .short('s')
                        .help("Split RINEX into two separate files"))
                .next_help_heading("RINEX output")
                    .arg(Arg::new("output")
                        .long("output")
                        .value_name("FILE")
                        .action(ArgAction::Append)
                        .help("Custom file paths to be generated from preprocessed RINEX files.
For example -fp was filtered and decimated, use --output to dump results into a new RINEX."))
                    .arg(Arg::new("custom-header")
                        .long("custom-header")
                        .value_name("JSON")
                        .action(ArgAction::Append)
                        .help("Custom header attributes, in case we're generating data.
--custom-header must either be plain JSON or an external JSON descriptor.
Refer to README"))
                    .get_matches()
            },
        }
    }
    /// Returns input filepaths
    pub fn input_path(&self) -> &str {
        self.matches.get_one::<String>("filepath").unwrap() // mandatory flag
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
    pub fn gf_recombination(&self) -> bool {
        self.matches.get_flag("gf")
    }
    pub fn wl_recombination(&self) -> bool {
        self.matches.get_flag("wl")
    }
    pub fn nl_recombination(&self) -> bool {
        self.matches.get_flag("nl")
    }
    pub fn mw_recombination(&self) -> bool {
        self.matches.get_flag("mw")
    }
    pub fn iono_detector(&self) -> bool {
        self.matches.get_flag("iono")
    }
    pub fn identification(&self) -> bool {
        self.matches.get_flag("sv")
            | self.matches.get_flag("epochs")
            | self.matches.get_flag("header")
            | self.matches.get_flag("observables")
            | self.matches.get_flag("ssi-range")
            | self.matches.get_flag("orbits")
            | self.matches.get_flag("nav-msg")
            | self.matches.get_flag("anomalies")
    }
    /// Returns true if Sv accross epoch display is requested
    pub fn sv_epoch(&self) -> bool {
        self.matches.get_flag("sv-epoch")
    }
    /// Epoch interval (histogram) analysis
    pub fn sampling_histogram(&self) -> bool {
        self.matches.get_flag("sampling-hist")
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
        let flags = vec![
            "sv",
            "epochs",
            "header",
            "constellations",
            "observables",
            "ssi-range",
            "ssi-sv-range",
            "orbits",
            "nav-msg",
            "anomalies",
        ];
        flags
            .iter()
            .filter(|x| self.matches.get_flag(x))
            .map(|x| *x)
            .collect()
    }
    fn get_flag(&self, flag: &str) -> bool {
        self.matches.get_flag(flag)
    }
    /// returns true if pretty JSON is requested
    pub fn pretty(&self) -> bool {
        self.get_flag("pretty")
    }
    /// Returns true if quiet mode is activated
    pub fn quiet(&self) -> bool {
        self.matches.get_flag("quiet")
    }
    /// Returns true if position solver is enabled
    pub fn rtk(&self) -> bool {
        self.matches.get_flag("rtk") || self.forced_spp() || self.forced_ppp()
    }
    /// Returns true if position solver forced to SPP
    pub fn forced_spp(&self) -> bool {
        self.matches.get_flag("spp")
    }
    /// Returns true if position solver forced to PPP
    pub fn forced_ppp(&self) -> bool {
        self.matches.get_flag("spp")
    }
    pub fn rtk_only(&self) -> bool {
        self.matches.get_flag("rtk-only")
    }
    pub fn cs_graph(&self) -> bool {
        self.matches.get_flag("cs")
    }
    /*
     * Returns possible file path to merge
     */
    pub fn merge_path(&self) -> Option<&Path> {
        self.matches
            .get_one::<String>("merge")
            .and_then(|s| Some(Path::new(s)))
    }
    /// Returns optionnal RINEX file to "merge"
    pub fn to_merge(&self) -> Option<Rinex> {
        if let Some(path) = self.merge_path() {
            let path = path.to_str().unwrap();
            if let Ok(rnx) = Rinex::from_file(path) {
                Some(rnx)
            } else {
                error!("failed to parse \"{}\"", path);
                None
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
    /*
     * Returns possible list of directories passed as specific data pool
     */
    pub fn data_directories(&self, key: &str) -> Vec<ReadDir> {
        if let Some(matches) = self.matches.get_many::<String>(key) {
            matches
                .filter_map(|s| {
                    let path = Path::new(s.as_str());
                    if path.is_dir() {
                        if let Ok(rd) = path.read_dir() {
                            Some(rd)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }
    /*
     * Returns possible list of files to be loaded individually
     */
    pub fn data_files(&self, key: &str) -> Vec<String> {
        if let Some(matches) = self.matches.get_many::<String>(key) {
            matches
                .filter_map(|s| {
                    let path = Path::new(s.as_str());
                    if path.is_file() {
                        Some(path.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
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
            let content: Vec<&str> = args.split(",").collect();
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
            let content: Vec<&str> = args.split(",").collect();
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
