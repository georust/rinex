use chrono::NaiveDateTime;
use clap::{Parser, Subcommand};

#[derive(Debug, Clone)]
#[derive(Subcommand)]
pub enum Commands {
    /// Extraction commands 
    Extract {
        #[arg(long, value_parser)]
        /// Header section
        header: bool,
        #[arg(short, long, value_parser)]
        /// Identified constellations 
        constellations: bool,
        #[arg(short, long, value_parser)]
        /// Encountered space vehicules 
        sv: bool,
        #[arg(long, value_parser)]
        /// Encountered space vehicules per epoch 
        sv_epoch: bool,
        #[arg(short, long, value_parser)]
        /// Encountered epochs 
        epochs: bool,
        #[arg(short, long, value_parser)]
        /// Unexpected data gaps 
        gaps: bool,
        #[arg(short, long, value_parser)]
        /// Largest unexpected data gap
        largest_gaps: bool,
    },
    /// Observation specific operations 
    Observation {
        #[arg(short, long, value_parser)]
        /// Extract identified observables 
        obs: bool,
        #[arg(short, long, value_parser)]
        /// Extract SSI (min, max) value from Observation RINEX 
        ssi_range: bool,
        #[arg(long, value_parser)]
        /// Extract SSI (min, max) per vehicule
        ssi_sv_range: bool,
        #[arg(short, long, value_parser)]
        /// Extract receiver clock offsets per epoch
        clock_offsets: bool,
        #[arg(long, value_parser)]
        /// Extract possible cycle slip events per epoch
        cycle_slips: bool,
        #[arg(short, long, value_parser)]
        /// Converts all Pseudo Range raw data into real physical distances 
        pr2distance: bool,
    },
    /// Navigation specific operations 
    Navigation {
        #[arg(short, long, value_parser)]
        /// Extract identified orbit fields 
        orb: bool,
        #[arg(long, value_parser)]
        /// Extract Clock biases
        biases: bool,
    },
    /// Record resampling
    Resampling {
        #[arg(short, long, value_parser)]
        /// Decimation ratio.
        /// For example, ratio = 2 will retain 1 epoch out of 2
        /// a,b,c,d => a,c
        ratio: Option<u64>,
        #[arg(short, long)]
        /// Decimation interval.
        /// Removes all epoch in between |(e(k) - e(k-1)| < interval
        /// predicate
        interval: Option<String>,
        #[arg(short, long)]
        /// Epoch window, discards all epochs
        /// that do not lie within the a < x < b interval
        window: Option<String>,
    },
    /// Retain filter list
    Retain {
        #[arg(short, long)]
        /// List of GNSS constellation to retain
        constellations: Option<String>,
        #[arg(short, long)]
        /// List of space vehicules to retain
        sv: Option<String>,
        #[arg(short, long)]
        /// Retain only valid epochs
        epoch_ok: bool, 
        #[arg(long)]
        /// Retain only non valid epochs
        epoch_nok: bool,
        #[arg(short, long)]
        /// List of Observables to retain.
        obs: Option<String>,
        #[arg(short, long)]
        /// Retain Legacy NAV only 
        lnav: bool, 
        #[arg(short, long)]
        /// Retain Modern (non Legacy) NAV only 
        mnav: bool, 
        #[arg(short, long)]
        /// List of Navigation messages to retain 
        nav_msg: Option<String>,
        #[arg(long)]
        /// List of Navigation Orbits to retain
        orb: Option<String>,
    },
    /// Filter data out
    Filter {
        /// SSI condition filter on retained Observations
        ssi: Option<u8>,
        /// LLI AND() mask on retained Observations
        lli_mask: Option<u8>,
    },
    /// RINEX production context
    Production {
        #[arg(short, long)]
        /// Output file name 
        filepath: Option<String>,
        #[arg(short, long)]
        /// Custom header fields
        header: Option<String>,
    },
    /// TEQC operations
    Teqc {
        #[arg(short, long)]
        /// Merge two RINEX files into one
        merge: bool,
        #[arg(short, long)]
        /// Split RINEX file into two 
        split: bool,
        #[arg(short, long)]
        /// Tiny ascii plot
        plot: bool,
        #[arg(short, long)]
        /// `teqc` verbose report and analysis 
        report: bool,
    },
    /// RINEX processing operations
    Processing {
        #[arg(short, long)]
        /// Computes single Observation RINEX differentiation,
        /// to cancel ionospheric biases.
        /// Requires two Observation RINEX files.
        diff: bool,
        #[arg(short, long)]
        /// Computes double Observation RINEX differentiation,
        /// to cancel ionospheric biases and clock induced biases.
        /// Requires two Observation RINEX and one Navigation file.
        double_diff: bool,
    },
}

#[derive(Parser, Debug)]
#[derive(Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Generate plot instead of stdout output
    #[clap(short, long, value_parser)]
    plot: bool,
    /// Pretty stdout output 
    #[clap(long, value_parser)]
    pretty: bool,
    /// Known commands 
    #[command(subcommand)]
    pub commands: Option<Commands>,
}

pub struct Cli {
    /// Arguments passed by user
    pub args: Args,
}

impl Cli {
    pub fn new() -> Self {
        Self {
            args: Args::parse(),
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
