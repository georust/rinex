// sub opmode
use clap::{value_parser, Arg, ArgAction, Command};
use std::path::PathBuf;

pub fn subcommand() -> Command {
    Command::new("sub")
        .long_flag("sub")
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
}
