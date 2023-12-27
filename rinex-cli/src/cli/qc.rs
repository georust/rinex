// tbin opmode
use clap::{value_parser, Arg, ArgAction, ArgMatches, ColorChoice, Command};
use rinex::prelude::Duration;

pub fn subcommand() -> Command {
    Command::new("qc")
        .short_flag('Q')
        .long_flag("qc")
        .about(
            "File Quality analysis (statistical evaluation) of the dataset.
This is typically used prior precise point positioning.",
        )
        .arg(
            Arg::new("cfg")
                .long("cfg")
                .required(false)
                .value_name("FILE")
                .action(ArgAction::Append)
                .help("Pass a QC configuration file (JSON)."),
        )
}
