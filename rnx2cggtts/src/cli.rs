use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};
use gnss_rtk::prelude::RTKConfig;
use log::{error, info};
use rinex::prelude::*;
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
and added like they were individually imported with -f."))
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
The default workspace is cggtts/workspace"))
                    .arg(Arg::new("graph")
                        .short('g')
                        .long("graph")
                        .action(ArgAction::SetTrue)
                        .help("Disable graphs generation, only text reports are to be generated."))
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
                    .arg(Arg::new("spp")
                        .long("spp")
                        .action(ArgAction::SetTrue)
                        .help("Enables Positioning forced to Single Frequency SPP solver mode.
Disregards whether the provided context is PPP compatible. 
NB: we do not account for Relativistic effects by default and raw pseudo range are used.
For indepth customization, refer to the configuration file and online documentation."))
                    .get_matches()
            },
        }
    }
    /// Returns input base dir
    pub fn input_base_dir(&self) -> Option<&String> {
        self.matches.get_one::<String>("directory")
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
    /// returns true if pretty JSON is requested
    pub fn pretty(&self) -> bool {
        self.get_flag("pretty")
    }
    /// Returns true if quiet mode is activated
    pub fn quiet(&self) -> bool {
        self.matches.get_flag("quiet")
    }
    /// Returns true if position solver forced to SPP
    pub fn forced_spp(&self) -> bool {
        self.matches.get_flag("spp")
    }
    /// Returns true if position solver forced to PPP
    pub fn forced_ppp(&self) -> bool {
        self.matches.get_flag("spp")
    }
    /*
     * graphs to be generated
     */
    pub fn graph(&self) -> bool {
        self.matches.get_flag("graph")
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
