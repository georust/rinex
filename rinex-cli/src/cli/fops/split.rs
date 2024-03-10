// Merge opmode
use clap::{value_parser, Arg, ArgAction, Command};
use rinex::prelude::Epoch;

use super::{SHARED_DATA_ARGS, SHARED_GENERAL_ARGS};

pub fn subcommand() -> Command {
    Command::new("split")
        .short_flag('s')
        .long_flag("split")
        .arg_required_else_help(true)
        .about("Split input file(s) at specified Epoch")
        .arg(
            Arg::new("split")
                .value_parser(value_parser!(Epoch))
                .value_name("EPOCH")
                .action(ArgAction::Set)
                .required(true)
                .help("Epoch (instant) to split at."),
        )
        .next_help_heading("Production Environment")
        .args(SHARED_GENERAL_ARGS.iter())
        .next_help_heading("Data context")
        .args(SHARED_DATA_ARGS.iter())
}
