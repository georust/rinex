// Merge opmode
use clap::{value_parser, Arg, ArgAction, ArgMatches, ColorChoice, Command};
use rinex::prelude::Epoch;
use std::path::PathBuf;

pub fn subcommand() -> Command {
    Command::new("split")
        .short_flag('s')
        .long_flag("split")
        .arg_required_else_help(true)
        .about("Split input RINEX files at specified Epoch.")
        .arg(
            Arg::new("split")
                .value_parser(value_parser!(Epoch))
                .value_name("EPOCH")
                .action(ArgAction::Set)
                .required(true)
                .help("Epoch (instant) to split at."),
        )
}
