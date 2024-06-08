// tbin opmode
use clap::{value_parser, Arg, ArgAction, Command};
use rinex::prelude::Duration;

use super::{SHARED_DATA_ARGS, SHARED_GENERAL_ARGS};

pub fn subcommand() -> Command {
    Command::new("tbin")
        .long_flag("tbin")
        .arg_required_else_help(true)
        .about("Time binning. Split RINEX files into a batch of equal duration.")
        .arg(
            Arg::new("interval")
                .value_parser(value_parser!(Duration))
                .value_name("Duration")
                .action(ArgAction::Set)
                .required(true)
                .help("Duration"),
        )
        .next_help_heading("Production Environment")
        .args(SHARED_GENERAL_ARGS.iter())
        .next_help_heading("Data context")
        .args(SHARED_DATA_ARGS.iter())
}
