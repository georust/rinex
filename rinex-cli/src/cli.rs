use clap::{
    Command, 
    Arg, ArgMatches, 
    //ArgGroup, 
    ArgAction,
};

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
                        //TODO to support several files @ once
                        //.action(ArgAction::Append)
                        //.value_terminator(",")
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
                    .arg(Arg::new("lock-loss")
                        .long("lock-loss")
                        .action(ArgAction::SetTrue)
                        .help("List epochs where lock was declared lost")) 
                    .arg(Arg::new("pr2distance")
                        .long("pr2distance")
                        .action(ArgAction::SetTrue)
                        .help("Converts all Pseudo Range data to real physical distances. This is destructive, original pseudo ranges are lost and overwritten"))
                    .arg(Arg::new("orbits")
                        .long("orbits")
                        .action(ArgAction::SetTrue)
                        .help("List identified orbits data fields. Applies to Navigation RINEX"))
                    .arg(Arg::new("nav-msg")
                        .long("nav-msg")
                        .action(ArgAction::SetTrue)
                        .help("List identified Navigation frame types")) 
                    .arg(Arg::new("elevation")
                        .long("elevation")
                        .action(ArgAction::SetTrue)
                        .help("Display elevation angles, per vehicules accross all epochs"))
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
                        .help("Downsample record content by given factor. 2 for instance, keeps one every other epoch"))
                    .arg(Arg::new("resample-interval")
                        .long("resample-interval")
                        .short('i')
                        .help("Discards every epoch in between |e(n)-(n-1)| < interval, where interval is a valid \"chrono::Duration\" string description"))
                    .arg(Arg::new("time-window")
                        .long("time-window")
                        .short('w')
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
                    .arg(Arg::new("retain-nav-eph")
                        .long("retain-nav-eph")
                        .help("Retains only Navigation ephemeris frames")) 
                    .arg(Arg::new("retain-nav-iono")
                        .long("retain-nav-iono")
                        .help("Retains only Navigation ionospheric models")) 
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
                        .action(ArgAction::SetTrue)
                        .help("Print (\"stdout\") a tiny ascii plot, similar to \"teqc\""))
                    .arg(Arg::new("teqc-report")
                        .long("teqc-report")
                        .action(ArgAction::SetTrue)
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
                        .action(ArgAction::SetTrue)
                        .help("Generate Plots instead of default \"stdout\" terminal output"))
                    .arg(Arg::new("pretty")
                        .long("pretty")
                        .action(ArgAction::SetTrue)
                        .help("Make \"stdout\" terminal output more readable"))
                    .get_matches()
            },
        }
    }
    /// Returns input filepath
    pub fn input_filepath(&self) -> &str {
        self.matches
            .get_one::<String>("filepath")
            .unwrap()
    }
    /// Returns output filepath
    pub fn output_filepath(&self) -> Option<&str> {
        self.matches
            .get_one::<String>("output-file").map(|x| x.as_str())
    }
    /// Returns true if at least one --extract category flag
    /// was requested
    pub fn extract(&self) -> bool {
        self.matches.get_flag("sv")
        | self.matches.get_flag("epochs")
        | self.matches.get_flag("sv-epoch")
        | self.matches.get_flag("header")
        | self.matches.get_flag("observables")
        | self.matches.get_flag("clock-offset")
        | self.matches.get_flag("ssi-range")
        | self.matches.get_flag("ssi-sv-range")
        | self.matches.get_flag("cycle-slip")
        | self.matches.get_flag("orbits")
        | self.matches.get_flag("elevation")
        | self.matches.get_flag("nav-msg")
        | self.matches.get_flag("clock-bias")
        | self.matches.get_flag("gaps")
        | self.matches.get_flag("largest-gap")
        | self.matches.get_flag("lock-loss")
    }
    /// Returns list of requested data to extract
    pub fn extraction_ops(&self) -> Vec<&str> {
        let flags = vec![
            "sv",
            "sv-epoch",
            "epochs",
            "header",
            "constellations",
            "observables",
            "clock-offset",
            "ssi-range",
            "ssi-sv-range",
            "cycle-slip",
            "orbits",
            "elevation",
            "nav-msg",
            "clock-bias",
            "gaps",
            "largest-gap",
            "lock-loss",
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
    }
    /// Returns list of retain ops to perform with given list of arguments
    pub fn retain_ops(&self) -> Vec<(&str, Vec<&str>)> {
        // this list order is actually important,
        //   because they describe the filter operation order
        //   it is better to have epochs filter first
        //    then the rest will follow
        let flags = vec![
            "retain-epoch-ok",
            "retain-epoch-nok",
            "retain-constell",
            "retain-sv",
            "retain-obs",
            "retain-ssi",
            "retain-orb",
            "retain-lnav",
            "retain-mnav",
            "retain-nav-msg",
            "retain-nav-eph",
            "retain-nav-iono",
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
    pub fn plot (&self) -> bool {
        self.get_flag("plot")
    }
}
