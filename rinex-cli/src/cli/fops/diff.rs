// sub opmode
use clap::{value_parser, Arg, ArgAction, Command};
use std::path::PathBuf;

use super::{SHARED_DATA_ARGS, SHARED_GENERAL_ARGS};

pub fn subcommand() -> Command {
    Command::new("diff")
        .arg_required_else_help(true)
        .about(
            "RINEX(A)-RINEX(B) substraction operation.
This is typically used to compare two GNSS receivers together.",
        )
        .arg(
            Arg::new("file")
                .value_parser(value_parser!(PathBuf))
                .value_name("FILEPATH")
                .action(ArgAction::Set)
                .required(true)
                .help(
                    "RINEX(B) to substract to a single RINEX file (A), that was previously loaded.",
                ),
        )
        .next_help_heading("Production Environment")
        .args(SHARED_GENERAL_ARGS.iter())
        .next_help_heading("Data context")
        .args(SHARED_DATA_ARGS.iter())
}
