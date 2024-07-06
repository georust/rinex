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
            Arg::new("nostats")
                .long("nostats")
                .action(ArgAction::SetTrue)
                .help("Disable statistical annotations (deviation, mean..)"),
        )
        .arg(
            Arg::new("force")
                .long("force")
                .action(ArgAction::SetTrue)
                .help("Force report regeneration.
Report is generated on first run (new command line options),
otherwise it is only incremented.
This option will force report re-generation, even if command line has not changed")
}
