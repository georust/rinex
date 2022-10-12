use chrono::NaiveDateTime;
use clap::{Parser, Command, Subcommand, Arg, ArgMatches, ArgGroup, ArgAction};

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
                    .arg(Arg::new("filepath")
                        .short('f')
                        .long("fp")
                        .help("Input RINEX file")
                        .required(true))
                    .arg(Arg::new("epochs")
                        .long("epochs")
                        .short('e')
                        .action(ArgAction::SetTrue)
                        .help("List identified epochs"))
                    .arg(Arg::new("constellations")
                        .long("constellations")
                        .short('c')
                        .action(ArgAction::SetTrue)
                        .help("List identified GNSS constellations"))
                    .arg(Arg::new("sv")
                        .short('s')
                        .long("sv")
                        .action(ArgAction::SetTrue)
                        .help("List identified space vehicules"))
                    .arg(Arg::new("sv-epoch")
                        .long("sv-epoch")
                        .action(ArgAction::SetTrue)
                        .help("List identified space vehicules per epoch"))
                    .arg(Arg::new("header")
                        .long("header")
                        .action(ArgAction::SetTrue)
                        .help("Extract header fields"))
                    .arg(Arg::new("observables")
                        .long("observables")
                        .short('o')
                        .action(ArgAction::SetTrue)
                        .help("List identified observables. Applies to Observation and Meteo RINEX"))
                    .arg(Arg::new("ssi-range")
                        .long("ssi-range")
                        .action(ArgAction::SetTrue)
                        .help("Extract SSI (min,max) range, accross all epochs and vehicules"))
                    .arg(Arg::new("ssi-sv-range")
                        .long("ssi-sv-range")
                        .action(ArgAction::SetTrue)
                        .help("Extract SSI (min,max) range, per vehicule, accross all epochs"))
                    .arg(Arg::new("clock-offset")
                        .long("clock-offset")
                        .action(ArgAction::SetTrue)
                        .help("Extract clock offset data, per epoch")) 
                    .arg(Arg::new("cycle-slip")
                        .long("cycle-slip")
                        .action(ArgAction::SetTrue)
                        .help("List epochs where possible cycle slip happened")) 
                    .arg(Arg::new("pr2distance")
                        .long("pr2distance")
                        .action(ArgAction::SetTrue)
                        .help("Converts all Pseudo Range data to real physical distances. This is destructive, original pseudo ranges are lost and overwritten"))
                    .arg(Arg::new("orbits")
                        .long("orbits")
                        .action(ArgAction::SetTrue)
                        .help("List identified orbits data fields. Applies to Navigation RINEX"))
                    .arg(Arg::new("clock-bias")
                        .long("clock-bias")
                        .action(ArgAction::SetTrue)
                        .help("Extract clock biases (offset, drift, drift changes) per epoch and vehicule"))
                    .arg(Arg::new("gaps")
                        .long("gaps")
                        .short('g')
                        .action(ArgAction::SetTrue)
                        .help("Identify unexpected data gaps in record"))
                    .arg(Arg::new("largest-gap")
                        .long("largest-gap")
                        .action(ArgAction::SetTrue)
                        .help("Identify largest data gaps in record"))
                    .arg(Arg::new("resample-ratio")
                        .long("resample-ratio")
                        .short('r')
                        .action(ArgAction::SetTrue)
                        .help("Downsample record content by given factor. 2 for instance, keeps one every other epoch"))
                    .arg(Arg::new("resample-interval")
                        .long("resample-interval")
                        .short('i')
                        .action(ArgAction::SetTrue)
                        .help("Discards every epoch in between |e(n)-(n-1)| < interval, where interval is a valid \"chrono::Duration\" string description"))
                    .arg(Arg::new("time-window")
                        .long("time-window")
                        .short('w')
                        .action(ArgAction::SetTrue)
                        .help("Center record content to specified epoch window. All epochs that do not lie within the specified (start, end) interval are dropped out. User must pass two valid \"chrono::NaiveDateTime\" description"))
                    .arg(Arg::new("retain-constell")
                        .long("retain-constell")
                        .help("Retain only given GNSS constellation"))
                    .arg(Arg::new("retain-sv")
                        .long("retain-sv")
                        .help("Retain only given Space vehicules"))
                    .arg(Arg::new("retain-epoch-ok")
                        .long("retain-epoch-ok")
                        .help("Retain only valid epochs"))
                    .arg(Arg::new("retain-epoch-nok")
                        .long("retain-epoch-nok")
                        .help("Retain only non valid epochs"))
                    .arg(Arg::new("retain-obs")
                        .long("retain-obs")
                        .help("Retain only given list of Observables")) 
                    .arg(Arg::new("retain-ssi")
                        .long("retain-ssi")
                        .help("Retain only observations that have at least this signal quality"))
                    .arg(Arg::new("lli-mask")
                        .long("lli-mask")
                        .short('l')
                        .help("Apply LLI AND() mask to all observations. Also drops observations that did not come with an LLI flag"))
                    .arg(Arg::new("retain-orb")
                        .long("retain-orb")
                        .help("Retain only given list of Orbits fields")) 
                    .arg(Arg::new("retain-lnav")
                        .long("retain-lnav")
                        .help("Retain only Legacy Navigation frames")) 
                    .arg(Arg::new("retain-mnav")
                        .long("retain-mnav")
                        .help("Retain only Modern Navigation frames")) 
                    .arg(Arg::new("retain-nav-msg")
                        .long("retain-nav-msg")
                        .help("Retain only given list of Navigation messages")) 
                    .arg(Arg::new("output-file")
                        .long("output-file")
                        .help("Custom output file, in case we're generating data"))
                    .arg(Arg::new("custom-header")
                        .long("custom-header")
                        .help("Custom header attributes, in case we're generating data"))
                    .arg(Arg::new("merge")
                        .short('m')
                        .long("merge")
                        .help("Merge two RINEX files together"))
                    .arg(Arg::new("split")
                        .long("split")
                        .help("Split RINEX into two seperate files"))
                    .arg(Arg::new("teqc-plot")
                        .long("teqc-plot")
                        .help("Print (\"stdout\") a tiny ascii plot, similar to \"teqc\""))
                    .arg(Arg::new("teqc-report")
                        .long("teqc-report")
                        .help("Generate verbose report, similar to \"teqc\""))
                    .arg(Arg::new("diff")
                        .long("diff")
                        .help("Compute Observation RINEX differentiation to cancel ionospheric biases"))
                    .arg(Arg::new("ddiff")
                        .long("ddiff")
                        .help("Compute Observation RINEX double differentiation to cancel ionospheric and local clock induced biases"))
                    .arg(Arg::new("plot")
                        .short('p')
                        .long("plot")
                        .help("Generate Plots instead of default \"stdout\" terminal output"))
                    .arg(Arg::new("pretty")
                        .long("pretty")
                        .help("Make \"stdout\" terminal output more readable"))
                    .get_matches()
            },
        }
    }
/*
    /// Returns True if user requested a single file operation
    pub fn single_file_op(&self) -> bool {
        !self.double_file_op()
    }
    /// Returns True if user requested an operation that involves 2 files
    pub fn double_file_op(&self) -> bool {
        self.matches.is_present("merge")
            | self.matches.is_present("diff")
                | self.matches.is_present("ddiff")
    }
    /// Returns true if graphical view was requested
    pub fn plotting(&self) -> bool {
        self.matches.is_present("plot")
    }
    /// Returns true if improved terminal output was requested
    pub fn pretty_stdout(&self) -> bool {
        self.matches.is_present("pretty")
    }

    pub fn is_present(&self, arg: &str) -> bool {
        self.matches.is_present(arg)
    }

    /// Returns true if we should expose record content.
    /// We expose record content if user did not request at least 1 specific operation
    pub fn print_record(&self) -> bool {
        let ops = vec![
            "header",
            "obs",
            "sv", "sv-per-epoch",
            "ssi-range", "ssi-sv-range",
            "clock-offset",
            "clock-biases",
            "cycle-slips",
        ];
        self.matches
    }
*/
}
