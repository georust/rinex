// Positioning OPMODE
use clap::{value_parser, Arg, ArgAction, ArgMatches, ColorChoice, Command};

pub fn subcommand() -> Command {
    Command::new("positioning")
        .short_flag('p')
        .arg_required_else_help(false)
        .about("Post processed Positioning opmode.
Use this mode to resolve precise positions and local time from RINEX dataset.
Expectes Observation RINEX from a single (unique) receiver.")
        .arg(Arg::new("cfg")
            .short('c')
            .value_name("FILE")
            .required(false)
            .action(ArgAction::Append)
            .help("Pass a Position Solver configuration file (JSON).
        [https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/struct.Config.html] is the structure to represent. Format is JSON. 
        See [] for meaningful examples."))
}
