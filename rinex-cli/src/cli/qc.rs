// tbin opmode
use clap::{Arg, ArgAction, Command};

pub fn subcommand() -> Command {
    Command::new("qc")
        .arg_required_else_help(false)
        .about(
            "File Quality analysis (statistical evaluation) of the dataset.
Typically used prior precise point positioning.",
        )
        .arg(
            Arg::new("cfg")
                .short('c')
                .long("cfg")
                .required(false)
                .value_name("FILE")
                .action(ArgAction::Append)
                .help(
                    "Pass a QC configuration file (JSON).
[] is the structure to represent in JSON.
See [] for meaningful examples.",
                ),
        )
}
