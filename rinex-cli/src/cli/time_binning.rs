// tbin opmode
use clap::{value_parser, Arg, ArgAction, Command};
use rinex::prelude::Duration;

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
}
