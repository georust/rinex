use log::{error, info};
use std::path::Path;
use std::str::FromStr;
use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};

pub struct Cli {
    /// Arguments passed by user
    matches: ArgMatches,
}

impl Cli {
    /// Build new command line interface
    pub fn new() -> Self {
        Self {
            matches: {
                Command::new("ubx2rnx")
                    .author("Guillaume W. Bres, <guillaume.bressaix@gmail.com>")
                    .version(env!("CARGO_PKG_VERSION"))
                    .about("RINEX generator from UBlox device")
                    // .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .arg(Arg::new("port")
                        .short('p')
                        .long("port")
                        .value_name("PORT")
                        .help("Set device port, default: \"/dev/ttyUSB0\""))
                    .arg(Arg::new("baud")
                        .short('b')
                        .long("baud")
                        .value_name("BAUDRATE")
                        .help("Set port baudrate, default: \"9600\""))
                    .arg(Arg::new("observation")
                        .short('o')
                        .long("obs")
                        .help("Generate RINEX Observation, disabled by default"))
                    .arg(Arg::new("navigation")
                        .short('n')
                        .long("nav")
                        .help("Generate RINEX Navigation, disabled by default"))
                    .get_matches()
            },
        }
    }
    /* returns device port to use */
    pub fn port(&self) -> String {
        if let Some(p) = self.matches.get_one::<String>("port") {
            p.clone()
        } else {
            String::from("/dev/ttyUSB0")
        }
    }
    /* returns baudrate to use */
    pub fn baudrate(&self) -> Result<u32, std::num::ParseIntError> {
        if let Some(p) = self.matches.get_one::<String>("baudrate") {
            p.parse::<u32>()   
        } else {
            Ok(9600) 
        }
    }
    /* returns true if Observation Data to be generated */
    pub fn observation(&self) -> bool {
        self.matches.get_flag("observation")
    }
    /* returns true if Navigation Data to be generated */
    pub fn navigation(&self) -> bool {
        self.matches.get_flag("navigation")
    }
}
