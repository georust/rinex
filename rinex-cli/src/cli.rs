use crate::fops::filename;
use crate::parser::parse_epoch;
use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};
use log::{error, info, warn};
use rinex::{prelude::*, Merge, quality::QcOpts};
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
                        .help("Input RINEX file")
                        .action(ArgAction::Append)
                        .required(true))
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
                        .help("Plot Sv against Epoch.
Useful graph to determine vehicles of interest for specific operations.
When both `--fp` and Navigation context (`--nav`) were provided, this 
depicts shared epochs and vehicles between the two contexts."))
                    .arg(Arg::new("epoch-hist")
                        .long("epoch-hist")
                        .action(ArgAction::SetTrue)
                        .help("Epoch duration histogram analysis."))
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
                    .arg(Arg::new("sbas-filter")
                        .short('S')
                        .action(ArgAction::SetTrue)
                        .help("Filters out all SBAS vehicles"))
					.arg(Arg::new("preprocessing")
						.short('P')
						.num_args(1..)
						.help("Apply a preprocessing filter. Refer to filter design section of the README."))
                    .arg(Arg::new("resample-ratio")
                        .long("resample-ratio")
                        .short('r')
                        .value_name("RATIO(u32)")
                        .help("Downsample record content by given factor. 
For example, \"--resample-ratio 2\" would keep one every other epoch"))
                    .arg(Arg::new("resample-interval")
                        .long("resample-interval")
                        .short('i')
                        .value_name("DURATION")
                        .help("Shrinks record so adjacent epochs match 
the |e(n)-e(n-1)| > interval condition. 
Interval must be a valid \"HH:MM:SS\" duration description.
Example: -i 00:10:00 will have all epochs spaced by at least 10 minutes."))
                    .arg(Arg::new("time-window")
                        .long("time-window")
                        .value_name("Epoch(1), Epoch(N)")
                        .short('w')
                        .help("Center record content around specified epoch window. 
All epochs that do not lie within the specified (start, end) 
interval are dropped out. User must pass two valid Datetime description. Epochs are specified in UTC timescale.
Example: -w \"2020-01-01 2020-01-02\" will restrict to 2020/01/01 midnight to 24hours.
Example: -w \"2020-01-01 00:00:00 2020-01-01 01:00:00\" will restrict the first hour."))
                .next_help_heading("Observation RINEX")
                    .arg(Arg::new("observables")
                        .long("observables")
                        .short('o')
                        .action(ArgAction::SetTrue)
                        .help("Identify observables. Applies to Observation and Meteo RINEX"))
                    .arg(Arg::new("ssi-range")
                        .long("ssi-range")
                        .action(ArgAction::SetTrue)
                        .help("Display SSI (min,max) range, accross all epochs and vehicules"))
                    .arg(Arg::new("ssi-sv-range")
                        .long("ssi-sv-range")
                        .action(ArgAction::SetTrue)
                        .help("Extract SSI (min,max) range, per vehicule, accross all epochs"))
                    .arg(Arg::new("lli-mask")
                        .long("lli-mask")
                        .help("Applies given LLI AND() mask. 
Also drops observations that did not come with an LLI flag"))
                    .arg(Arg::new("clock-offset")
                        .long("clk")
                        .action(ArgAction::SetTrue)
                        .help("Receiver Clock offset / drift analysis."))
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
                    .arg(Arg::new("orbits")
                        .long("orbits")
                        .action(ArgAction::SetTrue)
                        .help("Enumerate orbit fields."))
                    .arg(Arg::new("pos-ecef")
                        .long("--pos-ecef")
                        .value_name("\"x,y,z\" coordinates in ECEF [m]")
                        .help("Define the ground position manualy, in [m] ECEF system.
Some calculations require a reference position.
Ideally this information is contained in the file Header, but user can manually define them (superceeds)."))
                    .arg(Arg::new("pos-geo")
                        .long("--pos-geo")
                        .value_name("\"lat,lon,alt\" coordinates in ddeg [°]")
                        .help("Define the ground position manualy, in decimal degrees.
Some calculations require a reference position.
Ideally this information is contained in the file Header, but user can manually define them (superceeds)."))
                    .arg(Arg::new("nav-msg")
                        .long("nav-msg")
                        .action(ArgAction::SetTrue)
                        .help("Identify Navigation frame types. -fp must be a NAV file")) 
                    .arg(Arg::new("clock-bias")
                        .long("clock-bias")
                        .action(ArgAction::SetTrue)
                        .help("Display clock biases (offset, drift, drift changes) per epoch and vehicule.
-fp must be a NAV file"))
                .next_help_heading("Navigation Data")
                    .arg(Arg::new("nav")
                        .long("nav")
                        .num_args(1..)
                        .value_name("FILE")
                        .help("Augment `--fp` analysis with Navigation data.
Most useful when combined to Observation RINEX. Also enables the complete (full) `--qc` mode.")) 
                .next_help_heading("ANTEX / APC ")
                    .arg(Arg::new("--atx")
                        .long("atx")
                        .action(ArgAction::SetTrue)
                        .help("Local ANTEX file, allows APC corrections."))
                .next_help_heading("Quality Check (QC)")
                    .arg(Arg::new("qc")
                        .long("qc")
                        .action(ArgAction::SetTrue)
                        .help("Enable Quality Check (QC) mode.
Runs thorough analysis on provided RINEX data.
The summary report by default is integrated to the global HTML report."))
					.arg(Arg::new("qc-config")
						.long("qc-cfg")
						.value_name("[FILE]")
						.help("Pass a QC configuration file."))
                    .arg(Arg::new("qc-only")
                        .long("qc-only")
                        .action(ArgAction::SetTrue)
                        .help("Enables QC mode and ensures no other analysis are performed (quickest qc rendition)."))
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
                .next_help_heading("Terminal (options)")
                    .arg(Arg::new("quiet")
                        .short('q')
                        .action(ArgAction::SetTrue)
                        .help("Disable all terminal output. Disable auto HTML opener, on HTML rendering."))
                    .arg(Arg::new("pretty")
                        .short('p')
                        .long("pretty")
                        .action(ArgAction::SetTrue)
                        .help("Make terminal output more readable"))
                    .get_matches()
            },
        }
    }
    /// Returns input filepaths
    pub fn input_path(&self) -> &str {
        self.matches.get_one::<String>("filepath").unwrap() // mandatory flag
    }
    /// Returns output filepaths
    pub fn output_path(&self) -> Option<&str> {
        if let Some(args) = self.matches.get_one::<String>("output") {
            Some(&args)
        } else {
            None
        }
    }
    pub fn mask_filters(&self) -> Vec<&String> {
        if let Some(filters) = self.matches.get_many::<String>("mask-filter") {
            filters.collect()
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
			let s = std::fs::read_to_string(path)
				.expect(&format!("failed to read \"{}\"", path));
			let opts: QcOpts = serde_json::from_str(&s)
				.expect("faulty qc configuration");
			opts
		} else {
			QcOpts::default()
		}
	}
    pub fn quality_check_separate(&self) -> bool {
        self.matches.get_flag("qc-separate")
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
    pub fn epoch_histogram(&self) -> bool {
        self.matches.get_flag("epoch-hist")
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
    /// Returns true if at least one resampling op is to be performed
    pub fn resampling(&self) -> bool {
        self.matches.contains_id("resample-ratio")
            | self.matches.contains_id("resample-interval")
            | self.matches.contains_id("time-window")
    }
    pub fn resampling_ops(&self) -> Vec<(&str, &str)> {
        // this order describes eventually the order of filtering operations
        let flags = vec!["resample-ratio", "resample-interval", "time-window"];
        flags
            .iter()
            .filter(|x| self.matches.contains_id(x))
            .map(|id| {
                let args = self.matches.get_one::<String>(id).unwrap();
                (id, args.as_str())
            })
            .map(|(id, args)| (*id, args))
            .collect()
    }
    fn get_flag(&self, flag: &str) -> bool {
        self.matches.get_flag(flag)
    }
    /// returns true if --pretty was passed
    pub fn pretty(&self) -> bool {
        self.get_flag("pretty")
    }
    /// Returns true if quiet mode is activated
    pub fn quiet(&self) -> bool {
        self.matches.get_flag("quiet")
    }
    pub fn tiny_html(&self) -> bool {
        self.matches.get_flag("tiny-html")
    }
    pub fn cs_graph(&self) -> bool {
        self.matches.get_flag("cs")
    }
    /// Returns optionnal RINEX file to "merge"
    pub fn to_merge(&self) -> Option<Rinex> {
        let fp = self.matches.get_one::<String>("merge")?;
        if let Ok(rnx) = Rinex::from_file(&fp) {
            Some(rnx)
        } else {
            error!("failed to parse \"{}\"", filename(fp));
            None
        }
    }
    /// Returns split operation args
    pub fn split(&self) -> Option<Epoch> {
        if self.matches.contains_id("split") {
            if let Some(args) = self.matches.get_one::<String>("split") {
                if let Ok(epoch) = parse_epoch(args) {
                    Some(epoch)
                } else {
                    panic!("failed to parse [DATETIME]");
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    /// Returns optionnal Nav path, for enhanced capabilities
    fn nav_paths(&self) -> Vec<&String> {
        if let Some(paths) = self.matches.get_many::<String>("nav") {
            paths.collect()
        } else {
            Vec::new()
        }
    }
    /// Returns optionnal Navigation context
    pub fn nav_context(&self) -> Option<Rinex> {
        let mut nav_ctx: Option<Rinex> = None;
        let paths = self.nav_paths();
        for path in paths {
            if let Ok(rnx) = Rinex::from_file(&path) {
                if let Some(ref mut ctx) = nav_ctx {
                    let _ = ctx.merge_mut(&rnx);
                } else {
                    info!("--nav augmented mode");
                    nav_ctx = Some(rnx);
                }
            } else {
                error!("failed to parse navigation file \"{}\"", filename(&path));
            }
        }
        nav_ctx
    }
    fn atx_path(&self) -> Option<&String> {
        if self.matches.contains_id("atx") {
            self.matches.get_one::<String>("atx")
        } else {
            None
        }
    }
    pub fn atx_context(&self) -> Option<Rinex> {
        if let Some(path) = self.atx_path() {
            if let Ok(rnx) = Rinex::from_file(&path) {
                if rnx.is_antex_rinex() {
                    info!("--atx context provided");
                    return Some(rnx);
                } else {
                    warn!("--atx should be antenna rinex file");
                }
            } else {
                error!("failed to parse atx file \"{}\"", filename(&path));
            }
        }
        None
    }
	fn manual_ecef(&self) -> Option<&String> {
		self.matches.get_one::<String>("pos-ecef")
	}
	fn manual_geodetic(&self) -> Option<&String> {
		self.matches.get_one::<String>("pos-geo")
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
