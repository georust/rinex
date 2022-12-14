use crate::fops::filename;
use crate::parser::parse_epoch;
use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};
use log::{error, info, warn};
use rinex::{navigation::ElevationMask, prelude::*};
use std::str::FromStr;

pub struct Cli {
    /// product prefix
    pub prefix: String,
    /// primary RINEX
    pub primary_rinex: Rinex,
    /// subsidary Navigation RINEX
    pub nav_rinex: Option<Rinex>,
    /// position extrapolated from given context(s)
    pub ground_position: Option<(f64, f64, f64)>,
    /// Arguments passed by user
    matches: ArgMatches,
}

impl Cli {
    /// Build new command line interface
    pub fn new() -> Self {
        Self {
            primary_rinex: Rinex::default(),
            prefix: String::default(),
            nav_rinex: None,
            ground_position: None, // to be determined later on
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
                .next_help_heading("RINEX identification commands")
                    .arg(Arg::new("epochs")
                        .long("epochs")
                        .action(ArgAction::SetTrue)
                        .help("Identify epochs"))
                    .arg(Arg::new("constellations")
                        .long("constellations")
                        .short('c')
                        .action(ArgAction::SetTrue)
                        .help("Identify GNSS constellations"))
                    .arg(Arg::new("sv")
                        .long("sv")
                        .action(ArgAction::SetTrue)
                        .help("Identify space vehicules"))
                    .arg(Arg::new("sv-epoch")
                        .long("sv-epoch")
                        .action(ArgAction::SetTrue)
                        .help("Plots encountered space vehicules per epoch.
Useful graph to determine unexpected and determine vehicules of interest, inside this record.
When both `--fp` and extra Navigation Context (`--nav`) are provided,
this emphasizes epochs where vehicules were sampled on both contexts."))
                    .arg(Arg::new("epoch-hist")
                        .long("epoch-hist")
                        .action(ArgAction::SetTrue)
                        .help("Epoch duration histogram (graphical) analysis."))
                    .arg(Arg::new("header")
                        .long("header")
                        .action(ArgAction::SetTrue)
                        .help("Extract header fields"))
                .next_help_heading("Record resampling methods")
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
                .next_help_heading("Retain filters (focus on data of interest)")
                    .arg(Arg::new("gps-filter")
                        .short('G')
                        .action(ArgAction::SetTrue)
                        .help("Filter all GPS vehicles out"))
                    .arg(Arg::new("glo-filter")
                        .short('R')
                        .action(ArgAction::SetTrue)
                        .help("Filter all Glonass vehicles out"))
                    .arg(Arg::new("gal-filter")
                        .short('E')
                        .action(ArgAction::SetTrue)
                        .help("Filter all Galileo vehicles out"))
                    .arg(Arg::new("bds-filter")
                        .short('C')
                        .action(ArgAction::SetTrue)
                        .help("Filter all BeiDou vehicles out"))
                    .arg(Arg::new("qzss-filter")
                        .short('J')
                        .action(ArgAction::SetTrue)
                        .help("Filter all QZSS vehicles out"))
                    .arg(Arg::new("sbas-filter")
                        .short('S')
                        .action(ArgAction::SetTrue)
                        .help("Filter all SBAS vehicles out"))
                    .arg(Arg::new("retain-constell")
                        .long("retain-constell")
                        .value_name("list(Constellation)")
                        .help("Retain only given GNSS constellation"))
                    .arg(Arg::new("retain-sv")
                        .long("retain-sv")
                        .value_name("list(Sv)")
                        .help("Retain only given Space vehicules"))
                    .arg(Arg::new("retain-epoch-ok")
                        .long("retain-epoch-ok")
                        .action(ArgAction::SetTrue)
                        .help("Retain only epochs where associated flag is \"Ok\" or \"Unknown\".
Truly applies to Observation RINEX only."))
                    .arg(Arg::new("retain-epoch-nok")
                        .long("retain-epoch-nok")
                        .action(ArgAction::SetTrue)
                        .help("Retain only non valid epochs.
Truly applies to Observation RINEX only."))
                    .arg(Arg::new("elev-mask")
                        .short('e')
                        .long("elev-mask")
                        .help("Apply given elevation mask.
Example: --elev-mask '>30' will retain Sv above 30°.
Example: --elev-mask '<=40' will retain Sv below 40° included."))
                .next_help_heading("Observation RINEX")
                    .arg(Arg::new("observables")
                        .long("observables")
                        .short('o')
                        .action(ArgAction::SetTrue)
                        .help("Identify observables. Applies to Observation and Meteo RINEX"))
                    .arg(Arg::new("retain-obs")
                        .long("retain-obs")
                        .value_name("List(Observables)")
                        .help("Retain only given list of Observables")) 
                    .arg(Arg::new("retain-phase")
                        .long("retain-phase")
                        .action(ArgAction::SetTrue)
                        .help("Retain only Phase Observations (all carriers)")) 
                    .arg(Arg::new("retain-doppler")
                        .long("retain-doppler")
                        .action(ArgAction::SetTrue)
                        .help("Retain only Doppler Observation (all carriers)")) 
                    .arg(Arg::new("retain-pr")
                        .long("retain-pr")
                        .action(ArgAction::SetTrue)
                        .help("Retain only Pseudo Range Observations (all carriers)")) 
                    .arg(Arg::new("retain-ssi")
                        .long("retain-ssi")
                        .help("Retain only observations that have at least this signal quality"))
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
                        .long("clock-offset")
                        .action(ArgAction::SetTrue)
                        .help("Plot receiver clock offsets per epoch."))
                    .arg(Arg::new("gf")
                        .long("gf")
                        .action(ArgAction::SetTrue)
                        .help("Visualize Geometry Free recombination of both Phase and PR measurements."))
                    .arg(Arg::new("wl")
                        .long("wl")
                        .action(ArgAction::SetTrue)
                        .help("Visualize Wide Lane recombination of both Phase and PR measurements."))
                    .arg(Arg::new("nl")
                        .long("nl")
                        .action(ArgAction::SetTrue)
                        .help("Visualize Narrow Lane recombination of both Phase and PR measurements."))
                    .arg(Arg::new("mw")
                        .long("mw")
                        .action(ArgAction::SetTrue)
                        .help("Visualize Melbourne-Wübbena recombinations"))
                    .arg(Arg::new("dcb")
                        .long("dcb")
                        .action(ArgAction::SetTrue)
                        .help("Visualize Differential Code Bias analysis"))
                    .arg(Arg::new("multipath")
                        .long("mp")
                        .action(ArgAction::SetTrue)
                        .help("Visualize Code Multipath analysis"))
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
If you're just interested in CS information, you probably just want `-qc` instead."))
                .next_help_heading("Navigation RINEX")
                    .arg(Arg::new("orbits")
                        .long("orbits")
                        .action(ArgAction::SetTrue)
                        .help("Identify orbit fields."))
                    .arg(Arg::new("ref-pos")
                        .long("ref-pos")
                        .value_name("x,y,z coordinates [m] ECEF")
                        .help("Reference position in [m] ECEF system.
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
                    .arg(Arg::new("retain-orb")
                        .long("retain-orb")
                        .help("Retain only given list of Orbits fields.
For example, \"satPosX\" and \"satPosY\" are valid Glonass Orbit fields.
Applies to either -fp or -nav context"))
                    .arg(Arg::new("retain-lnav")
                        .long("retain-lnav")
                        .action(ArgAction::SetTrue)
                        .help("Retain only Legacy Navigation frames.
Applies to either -fp or -nav context"))
                    .arg(Arg::new("retain-mnav")
                        .long("retain-mnav")
                        .action(ArgAction::SetTrue)
                        .help("Retain only Modern Navigation frames.
Applies to either -fp or -nav context"))
                    .arg(Arg::new("retain-nav-msg")
                        .long("retain-nav-msg")
                        .action(ArgAction::SetTrue)
                        .help("Retain only given list of Navigation messages.
Applies to either -fp or -nav context"))
                    .arg(Arg::new("retain-nav-eph")
                        .long("retain-nav-eph")
                        .action(ArgAction::SetTrue)
                        .help("Retains only Navigation ephemeris frames.
Applies to either -fp or -nav context"))
                    .arg(Arg::new("retain-nav-iono")
                        .long("retain-nav-iono")
                        .action(ArgAction::SetTrue)
                        .help("Retains only Navigation ionospheric models. 
-fp must be a NAV file"))
                .next_help_heading("Navigation Context")
                    .arg(Arg::new("nav")
                        .long("nav")
                        .value_name("FILE")
                        .help("Augment `--fp` with related Navigation Context.
Most useful when combined to Observation RINEX. 
Enables full `--qc` summary."))
                .next_help_heading("Quality Check (QC)")
                    .arg(Arg::new("qc")
                        .long("qc")
                        .action(ArgAction::SetTrue)
                        .help("Enable Quality Check (QC) mode.
Runs thorough analysis on provided RINEX data.
The summary report by default is integrated to the global HTML report."))
                    .arg(Arg::new("qc-separate")
                        .long("qc-separate")
                        .action(ArgAction::SetTrue)
                        .help("Dump QC report in separate HTML"))
                    .arg(Arg::new("qc-only")
                        .long("qc-only")
                        .action(ArgAction::SetTrue)
                        .help("Disable all but QC analysis, for quicker QC summary generation"))
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
                .next_help_heading("Terminal options")
                    .arg(Arg::new("quiet")
                        .short('q')
                        .action(ArgAction::SetTrue)
                        .help("Disable all terminal output. Disable auto HTML opener, on HTML rendering."))
                    .arg(Arg::new("pretty")
                        .short('p')
                        .long("pretty")
                        .action(ArgAction::SetTrue)
                        .help("Make terminal output more readable"))
                .next_help_heading("HTML options")
                    .arg(Arg::new("tiny-html")
                        .long("tiny-html")
                        .action(ArgAction::SetTrue)
                        .help("Generates smaller HTML content, but slower to render in a web browser"))
                    .get_matches()
            },
        }
    }
    /// Returns input filepaths
    pub fn input_path(&self) -> &str {
        self.matches.get_one::<String>("filepath").unwrap()
    }
    /// Returns output filepaths
    pub fn output_path(&self) -> Option<&str> {
        if let Some(args) = self.matches.get_one::<String>("output") {
            Some(&args)
        } else {
            None
        }
    }
    pub fn elevation_mask(&self) -> Option<ElevationMask> {
        let args = self.matches.get_one::<String>("elev-mask")?;
        if let Ok(mask) = ElevationMask::from_str(args) {
            Some(mask)
        } else {
            println!("failed to parse elevation mask from \"{}\"", args);
            None
        }
    }
    pub fn quality_check(&self) -> bool {
        self.matches.get_flag("qc")
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
    /// Returns true if at least one retain filter should be applied
    pub fn retain(&self) -> bool {
        self.matches.contains_id("retain-constell")
            | self.matches.contains_id("retain-sv")
            | self.matches.contains_id("retain-epoch-ok")
            | self.matches.contains_id("retain-epoch-nok")
            | self.matches.contains_id("retain-obs")
            | self.matches.contains_id("retain-ssi")
            | self.matches.contains_id("retain-orb")
            | self.matches.contains_id("retain-lnav")
            | self.matches.contains_id("retain-mnav")
            | self.matches.contains_id("retain-nav-msg")
            | self.matches.contains_id("retain-nav-eph")
            | self.matches.contains_id("retain-nav-iono")
            | self.matches.contains_id("retain-phase")
            | self.matches.contains_id("retain-doppler")
            | self.matches.contains_id("retain-pr")
    }

    pub fn retain_flags(&self) -> Vec<&str> {
        let flags = vec![
            "retain-epoch-ok",
            "retain-epoch-nok",
            "retain-lnav",
            "retain-mnav",
            "retain-nav-msg",
            "retain-nav-eph",
            "retain-nav-iono",
            "retain-phase",
            "retain-doppler",
            "retain-pr",
        ];
        flags
            .iter()
            .filter_map(|x| {
                if self.matches.get_flag(x) {
                    Some(*x)
                } else {
                    None
                }
            })
            .collect()
    }
    /// Returns list of retain ops to perform with given list of arguments
    pub fn retain_ops(&self) -> Vec<(&str, Vec<&str>)> {
        // this list order is actually important,
        //   because they describe the filter operation order
        //   it is better to have epochs filter first
        //    then the rest will follow
        let flags = vec![
            "retain-constell",
            "retain-sv",
            "retain-obs",
            "retain-ssi",
            "retain-orb",
        ];
        flags
            .iter()
            .filter(|x| self.matches.contains_id(x))
            .map(|id| {
                let descriptor = self.matches.get_one::<String>(id).unwrap();
                let args: Vec<&str> = descriptor.split(",").collect();
                (id, args)
            })
            .map(|(id, args)| (*id, args))
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

    /// Returns true if at least one filter should be applied
    pub fn filter(&self) -> bool {
        self.matches.contains_id("lli-mask")
            || self.matches.contains_id("gps-filter")
            || self.matches.contains_id("glo-filter")
            || self.matches.contains_id("gal-filter")
            || self.matches.contains_id("bds-filter")
            || self.matches.contains_id("qzss-filter")
            || self.matches.contains_id("sbas-filter")
    }
    pub fn filter_ops(&self) -> Vec<(&str, &str)> {
        let flags = vec!["lli-mask"];
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
    fn nav_path(&self) -> Option<&str> {
        if self.matches.contains_id("nav") {
            if let Some(args) = self.matches.get_one::<String>("nav") {
                Some(&args)
            } else {
                None
            }
        } else {
            None
        }
    }
    /// Returns optionnal Navigation context
    pub fn nav_context(&self) -> Option<Rinex> {
        if let Some(path) = self.nav_path() {
            if let Ok(rnx) = Rinex::from_file(path) {
                if rnx.is_navigation_rinex() {
                    info!("--nav augmented mode enabled");
                    return Some(rnx);
                } else {
                    warn!("--nav must should be navigation data");
                }
            } else {
                error!("failed to parse navigation file \"{}\"", filename(path));
            }
        }
        None
    }
    /// Returns ECEF position passed by usuer
    pub fn manual_position(&self) -> Option<(f64, f64, f64)> {
        let args = self.matches.get_one::<String>("ref-pos")?;
        let content: Vec<&str> = args.split(",").collect();
        if let Ok(pos_x) = f64::from_str(content[0].trim()) {
            if let Ok(pos_y) = f64::from_str(content[1].trim()) {
                if let Ok(pos_z) = f64::from_str(content[2].trim()) {
                    return Some((pos_x, pos_y, pos_z));
                } else {
                    error!("pos(z) should be f64 ECEF [m]");
                }
            } else {
                error!("pos(y) should be f64 ECEF [m]");
            }
        } else {
            error!("pos(x) should be f64 ECEF [m]");
        }
        None
    }
}
