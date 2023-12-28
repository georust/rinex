// Merge opmode
use clap::{value_parser, Arg, ArgAction, Command};
use std::path::PathBuf;

pub fn subcommand() -> Command {
    Command::new("merge")
        .short_flag('m')
        .long_flag("merge")
        .arg_required_else_help(true)
        .about("Merge a RINEX into another and dump result.")
        .arg(
            Arg::new("file")
                .value_parser(value_parser!(PathBuf))
                .value_name("FILEPATH")
                .action(ArgAction::Set)
                .required(true)
                .help("RINEX file to merge."),
        )
}
