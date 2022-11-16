use clap::{
    Command, 
    Arg, ArgMatches, 
    ArgAction,
    ColorChoice,
};
use crate::parser::{
    parse_duration,
    parse_epoch,
};
use rinex::prelude::*;

pub struct Cli {
    /// Arguments passed by user
    pub matches: ArgMatches,
}

impl Cli {
    /// Build new command line interface 
    pub fn new() -> Self {
        Self {
            matches: {
                Command::new("rinex-cli")
                    .author("Guillaume W. Bres, <guillaume.bressaix@gmail.com>")
                    .version("1.0")
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
                        .short('e')
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
                        .value_name("START, END")
                        .short('w')
                        .help("Center record content around specified epoch window. 
All epochs that do not lie within the specified (start, end) 
interval are dropped out. User must pass two valid Datetime description. Epochs are specified in UTC timescale.
Example: -w \"2020-01-01 2020-01-02\" will restrict to 2020/01/01 midnight to 24hours.
Example: -w \"2020-01-01 00:00:00 2020-01-01 01:00:00\" will restrict the first hour."))
                .next_help_heading("Retain filters (focus on data of interest)")
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
                    .arg(Arg::new("retain-elev-above")
                        .long("retain-elev-above")
                        .value_name("LIMIT(f64)")
                        .help("Retain vehicules (strictly) above given elevation angle.
-fp must be a NAV file, or NAV context must be provided with -nav"))
                    .arg(Arg::new("retain-elev-below")
                        .long("retain-elev-below")
                        .value_name("LIMIT(f64)")
                        .help("Retain vehicules (strictly) below given elevation angle.
-fp must be a NAV file, or NAV context must be provided with -nav"))
                    .arg(Arg::new("retain-best-elev")
                        .long("retain-best-elev")
                        .action(ArgAction::SetTrue)
                        .help("Retain vehicules per epoch and per constellation, 
that exhibit the best elevation angle.
-fp must be a NAV file, or NAV context must be provided with -nav"))
                .next_help_heading("Observation RINEX specific (-fp)")
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
                        .short('l')
                        .help("Applies given LLI AND() mask. 
Also drops observations that did not come with an LLI flag"))
                    .arg(Arg::new("clock-offset")
                        .long("clock-offset")
                        .action(ArgAction::SetTrue)
                        .help("Plot receiver clock offsets per epoch."))
                    .arg(Arg::new("gf")
                        .long("gf")
                        .action(ArgAction::SetTrue)
                        .help("Request Geometry Free recombination of Phase and PR measurements. 
This serves as a CS indicator or atmospheric delay estimator. Refer to README."))
                    .arg(Arg::new("dcb")
                        .long("dcb")
                        .action(ArgAction::SetTrue)
                        .help("Differential Code Bias analysis (DCBs).
Useful to determine correlation and biases between Phase and PR observations.
For instance \"2S-2W\" means S code against W code, for L2 carrier. Refer to README."))
                    .arg(Arg::new("multipath")
                        .long("mp")
                        .action(ArgAction::SetTrue)
                        .help("Run code multipath analysis. Refer to README."))
                    .arg(Arg::new("lock-loss")
                        .long("lock-loss")
                        .action(ArgAction::SetTrue)
                        .help("Visualize which code might be affected by CS, accross all epochs."))
                    .arg(Arg::new("pr2distance")
                        .long("pr2distance")
                        .action(ArgAction::SetTrue)
                        .help("Converts all Pseudo Range data to real physical distances. 
This is destructive, original pseudo range codes are lost and overwritten"))
                .next_help_heading("Navigation RINEX specific")
                    .arg(Arg::new("orbits")
                        .long("orbits")
                        .action(ArgAction::SetTrue)
                        .help("Identify orbits data fields. -fp must be a NAV file"))
                    .arg(Arg::new("nav-msg")
                        .long("nav-msg")
                        .action(ArgAction::SetTrue)
                        .help("Identify Navigation frame types. -fp must be a NAV file")) 
                    .arg(Arg::new("elevation")
                        .long("elevation")
                        .action(ArgAction::SetTrue)
                        .help("Display elevation angles, per vehicules accross all epochs.
-fp must be a NAV file"))
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
                .next_help_heading("RINEX processing")
                    .arg(Arg::new("nav")
                        .long("nav")
                        .value_name("FILE")
                        .help("Provide Navigation context for advanced RINEX processing.
Usually combined to Observation data, provided with -fp.
Only identical epochs can be analyzed and processed.
Ideally, both contexts have strictly identical sample rates.
Refer to README."))
                .next_help_heading("`teqc` operations")
                    .arg(Arg::new("merge")
                        .short('m')
                        .value_name("FILE")
                        .long("merge")
                        .help("RINEX merge operation.
Combine this RINEX, considered secondary, into `--fp`. RINEX format must match."))
                    .arg(Arg::new("split")
                        .long("split")
                        .value_name("DATETIME")
                        .short('s')
                        .help("Split RINEX into two seperate files"))
                    .arg(Arg::new("ascii-plot")
                        .long("ascii-plot")
                        .action(ArgAction::SetTrue)
                        .help("Prints a tiny plot, similar to \"teqc\""))
                    .arg(Arg::new("qc")
                        .short('q')
                        .long("qc")
                        .action(ArgAction::SetTrue)
                        .help("RINEX quality check,
generates verbose report, similar to \"teqc\""))
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
                    .arg(Arg::new("pretty")
                        .long("pretty")
                        .action(ArgAction::SetTrue)
                        .help("Make terminal output more readable"))
                .next_help_heading("Data visualization")
                    .arg(Arg::new("skyplot")
                        .short('y')
                        .long("skyplot")
                        .action(ArgAction::SetTrue)
                        .help("Generate a \"skyplot\". NAV context must be provided, either with -fp or -nav"))
                    .arg(Arg::new("plot")
                        .short('p')
                        .long("plot")
                        .action(ArgAction::SetTrue)
                        .help("Generate Plots instead of default \"stdout\" terminal output"))
                    .arg(Arg::new("plot-width")
                        .long("plot-width")
                        .value_name("WIDTH(u32)")
                        .help("Set plot width. Default is 1024px.
Example \"--plot-width 2048"))
                    .arg(Arg::new("plot-height")
                        .long("plot-height")
                        .value_name("HEIGHT(u32)")
                        .help("Set plot height. Default is 768px.
Example \"--plot-height 1024"))
                    .arg(Arg::new("plot-dim")
                        .long("plot-dim")
                        .value_name("DIM(u32,u32)")
                        .help("Set plot dimensions. Example \"--plot-dim 2048,768\". Default is (1024, 768)px"))
                    .get_matches()
            },
        }
    }
    /// Returns input filepaths
    pub fn input_path(&self) -> &str {
        self.matches
            .get_one::<String>("filepath")
            .unwrap()
    }
    /// Returns output filepaths
    pub fn output_path(&self) -> Option<&str> {
        if let Some(args) = self.matches.get_one::<String>("output") {
            Some(&args)
        } else {
            None
        }
    }
    /// Returns true if GF recombination requested
    pub fn gf_recombination(&self) -> bool {
        self.matches.get_flag("gf")
    }
    /// Returns true if at least one basic identification flag was passed
    pub fn basic_identification(&self) -> bool {
        self.matches.get_flag("sv")
        | self.matches.get_flag("epochs")
        | self.matches.get_flag("header")
        | self.matches.get_flag("observables")
        | self.matches.get_flag("ssi-range")
        | self.matches.get_flag("orbits")
        | self.matches.get_flag("nav-msg")
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
        ];
        flags.iter()
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
        | self.matches.contains_id("retain-best-elev")
        | self.matches.contains_id("retain-elev-above")
        | self.matches.contains_id("retain-elev-below")
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
            "retain-best-elev",
        ];
        flags.iter()
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
            "retain-elev-above",
            "retain-elev-below",
            "retain-orb",
        ];
        flags.iter()
            .filter(|x| self.matches.contains_id(x))
            .map(|id| {
                let descriptor = self.matches.get_one::<String>(id)
                    .unwrap();
                let args: Vec<&str> = descriptor
                    .split(",")
                    .collect();
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
        let flags = vec![
            "resample-ratio",
            "resample-interval",
            "time-window",
        ];
        flags.iter()
            .filter(|x| self.matches.contains_id(x))
            .map(|id| {
                let args = self.matches.get_one::<String>(id)
                    .unwrap();
                (id, args.as_str())
            })
            .map(|(id, args)| (*id, args))
            .collect()
    }

    /// Returns true if at least one filter should be applied 
    pub fn filter(&self) -> bool {
        self.matches.contains_id("lli-mask")
    }
    pub fn filter_ops(&self) -> Vec<(&str, &str)> {
        let flags = vec![
            "lli-mask",
        ];
        flags.iter()
            .filter(|x| self.matches.contains_id(x))
            .map(|id| {
                let args = self.matches.get_one::<String>(id)
                    .unwrap();
                (id, args.as_str())
            })
            .map(|(id, args)| (*id, args))
            .collect()
    }
    fn get_flag (&self, flag: &str) -> bool {
        self.matches
            .get_flag(flag)
    }
    pub fn pretty (&self) -> bool {
        self.get_flag("pretty")
    }
    /// Returns optionnal RINEX file to "merge"
    pub fn merge(&self) -> Option<&str> {
        if self.matches.contains_id("merge") {
            if let Some(s) = self.matches.get_one::<String>("merge") {
                Some(&s)
            } else {
                None
            }
        } else {
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
    /// Returns optionnal Navigation context
    pub fn nav_context(&self) -> Option<Rinex> {
        if self.matches.contains_id("nav") {
            let args = self.matches.get_one::<String>("nav")
                .unwrap();
            if let Ok(rnx) = Rinex::from_file(args) {
                if rnx.is_navigation_rinex() {
                    Some(rnx)
                } else {
                    panic!("--nav must be a Navigation RINEX");
                }
            } else {
                println!("Failed to parse Navigation Context \"{}\"", args);
                None
            }
        } else {
            None
        }
    }
    pub fn plot (&self) -> bool {
        self.get_flag("plot")
    }
    /// Returns desired plot dimensions
    pub fn plot_dimensions(&self) -> (u32,u32) {
        let mut dim = (1024, 768);
        if self.matches.contains_id("plot-dim") {
            let args = self.matches.get_one::<String>("plot-dim")
                .unwrap();
            let items: Vec<&str> = args.split(",").collect();
            if items.len() == 2 {
                if let Ok(w) = u32::from_str_radix(items[0].trim(), 10) {
                    if let Ok(h) = u32::from_str_radix(items[1].trim(), 10) {
                        dim = (w, h);
                    }
                }
            }
        } else if self.matches.contains_id("plot-width") {
            let arg = self.matches.get_one::<String>("plot-width")
                .unwrap();
            if let Ok(w) = u32::from_str_radix(arg.trim(), 10) {
                dim.0 = w;
            }
        } else if self.matches.contains_id("plot-height") {
            let arg = self.matches.get_one::<String>("plot-height")
                .unwrap();
            if let Ok(h) = u32::from_str_radix(arg.trim(), 10) {
                dim.1 = h;
            }
        }
        dim
    }
}
