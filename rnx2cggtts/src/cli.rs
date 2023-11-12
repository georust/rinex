use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};
use log::{error, info};
use std::collections::HashMap;

use std::str::FromStr;

pub struct Cli {
    /// Arguments passed by user
    matches: ArgMatches,
}

use cggtts::{prelude::ReferenceTime, track::Scheduler};
use rinex::prelude::*;
use rtk::prelude::Config;

impl Cli {
    /// Build new command line interface
    pub fn new() -> Self {
        Self {
            matches: {
                Command::new("rnx2cggtts")
                    .author("Guillaume W. Bres, <guillaume.bressaix@gmail.com>")
                    .version(env!("CARGO_PKG_VERSION"))
                    .about("CGGTTS from RINEX Data generation tool")
                    .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .arg(Arg::new("filepath")
                        .short('f')
                        .long("fp")
                        .value_name("FILE")
                        .action(ArgAction::Append)
                        .required_unless_present("directory")
                        .help("Input RINEX file. Can be any kind of RINEX or SP3,
and you can load as many as you want."))
                    .arg(Arg::new("directory")
                        .short('d')
                        .long("dir")
                        .value_name("DIRECTORY")
                        .required_unless_present("filepath")
                        .help("Load directory recursively. RINEX and SP3 files are identified
and added like they were individually imported with -f.
You can load as many directories as you need."))
                    .arg(Arg::new("workspace")
                        .short('w')
                        .long("workspace")
                        .value_name("FOLDER")
                        .help("Customize workspace location (folder does not have to exist).
The default workspace is rinex-cli/workspace"))
                .next_help_heading("CGGTTS")
                    .arg(Arg::new("custom-clock")
                        .long("clk")
                        .value_name("NAME")
                        .help("Set the name of your local custom clock (in case it's a UTC replica, prefer -u)."))
                    .arg(Arg::new("utck")
                        .short('u')
                        .value_name("NAME")
                        .help("Set the name of your local UTC replica. In case your local clock is not tracking UTC, prefer --clk."))
                    .arg(Arg::new("station")
                        .short('s')
                        .value_name("NAME")
                        .help("Define / override the station name. If not specified, we expect the input
RINEX Observations to follow naming conventions and we deduce the station name from the filename."))
                    .arg(Arg::new("filename")
                        .short('o')
                        .value_name("FILENAME")
                        .help("Set CGGTTS filename to be generated (within workspace).
When not defined, the CGGTTS follows naming conventions, and is named after the Station and Receiver definitions."))
                .next_help_heading("Sky tracking & Common View")
                    .arg(Arg::new("tracking")
                        .short('t')
                        .value_name("DURATION")
                        .help("Modify tracking duration: default is 780s + 3' as defined by BIPM.
You can't modify the tracking duration unless you have complete control on both remote sites."))
                    .arg(Arg::new("single-sv")
                        .long("sv")
                        .value_name("SV")
                        .help("Track single (unique) Space Vehicle that must be in plain sight on both remote sites."))
                .next_help_heading("Setup / Hardware")
                    .arg(Arg::new("rfdly")
                        .long("rf-delay")
                        .action(ArgAction::Append)
                        .help("Specify the RF delay (frequency dependent), in nanoseconds.
Ideally, you should provide a time delay for all codes used by the solver.
For example, specify a 3.2 nanoseconds delay on C1 with: --rf-delay C1:3.2.")) 
                    .arg(Arg::new("refdly")
                        .long("ref-delay")
                        .help("Specify the delay between the GNSS receiver external clock and its local sampling clock. 
This is the delay induced by the cable on the external ref clock. Specify it in nanoseconds, for example: --ref-delay 5.0"))
                .next_help_heading("Solver")
					.arg(Arg::new("config")
						.long("cfg")
                        .short('c')
						.value_name("FILE")
						.help("Pass Positioning configuration, refer to README."))
                    .arg(Arg::new("spp")
                        .long("spp")
                        .action(ArgAction::SetTrue)
                        .help("Force solving strategy to SPP."))
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
						.help("Design preprocessing operations.
Refer to rinex-cli Preprocessor documentation for more information"))
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
    fn get_flag(&self, flag: &str) -> bool {
        self.matches.get_flag(flag)
    }
    pub fn spp(&self) -> bool {
        self.matches.get_flag("spp")
    }
    /// Returns the manualy defined RFDLY (in nanoseconds!)
    pub fn rf_delay(&self) -> Option<HashMap<Observable, f64>> {
        if let Some(delays) = self.matches.get_many::<String>("rfdly") {
            Some(
                delays
                    .into_iter()
                    .filter_map(|string| {
                        let items: Vec<_> = string.split(':').collect();
                        if items.len() < 2 {
                            error!("format error, command should be --rf-delay CODE:[nanos]");
                            None
                        } else {
                            let code = items[0].trim();
                            let nanos = items[0].trim();
                            if let Ok(code) = Observable::from_str(code) {
                                if let Ok(f) = nanos.parse::<f64>() {
                                    Some((code, f))
                                } else {
                                    error!("invalid nanos: expecting valid f64");
                                    None
                                }
                            } else {
                                error!(
                                    "invalid pseudo range CODE, expecting codes like \"L1C\",..."
                                );
                                None
                            }
                        }
                    })
                    .collect(),
            )
        } else {
            None
        }
    }
    /// Returns the manualy defined REFDLY (in nanoseconds!)
    pub fn reference_time_delay(&self) -> Option<f64> {
        if let Some(s) = self.matches.get_one::<String>("refdly") {
            if let Ok(f) = s.parse::<f64>() {
                info!("reference time delay manually defined");
                Some(f)
            } else {
                error!("reference time delay should be valid nanoseconds value");
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
    pub fn tracking_duration(&self) -> Duration {
        if let Some(t) = self.matches.get_one::<String>("tracking") {
            if let Ok(dt) = Duration::from_str(t.trim()) {
                warn!("using custom traking duration {}", dt);
                dt
            } else {
                panic!("incorrect tracking duration specification");
            }
        } else {
            Duration::from_seconds(Scheduler::BIPM_TRACKING_DURATION_SECONDS.into())
        }
    }
    fn utck(&self) -> Option<&String> {
        self.matches.get_one::<String>("utck")
    }
    fn custom_clock(&self) -> Option<&String> {
        self.matches.get_one::<String>("custom-clock")
    }
    /* reference time to use in header formatting */
    pub fn reference_time(&self) -> ReferenceTime {
        if let Some(utck) = self.utck() {
            ReferenceTime::UTCk(utck.clone())
        } else if let Some(clk) = self.custom_clock() {
            ReferenceTime::Custom(clk.clone())
        } else {
            ReferenceTime::Custom("Unknown".to_string())
        }
    }
    /* custom station name */
    pub fn custom_station(&self) -> Option<&String> {
        self.matches.get_one::<String>("station")
    }
    /* custom workspace */
    pub fn custom_workspace(&self) -> Option<&String> {
        self.matches.get_one::<String>("workspace")
    }
    /* custom filename */
    pub fn custom_filename(&self) -> Option<&String> {
        self.matches.get_one::<String>("filename")
    }
}
