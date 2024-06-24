// tbin opmode
use clap::{Arg, ArgAction, Command};

pub fn subcommand() -> Command {
    Command::new("qc")
        .arg_required_else_help(false)
        .about("RINEX and/or SP3 analysis")
        .long_about(
            "Use this mode to generate text/HTML based reports.
Reports will integrate Plots if application is compiled with `plot` feature.
Refer to online Wiki and scripts/ database for examples.",
        )
        .arg(
            Arg::new("cfg")
                .short('c')
                .long("cfg")
                .required(false)
                .value_name("FILE")
                .action(ArgAction::Append)
                .help("QC configuration file (JSON)"),
        )
        .arg(
            Arg::new("no-stat")
                .long("no-stat")
                .action(ArgAction::SetTrue)
                .help("Disable statistical annotation on some of the plots."),
        )
}
