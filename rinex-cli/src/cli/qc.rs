// tbin opmode
use clap::{Arg, ArgAction, Command};

pub fn subcommand() -> Command {
    Command::new("qc")
        .arg_required_else_help(false)
        .about(
            "Quality Check/Control analyzes GNSS data and generates HTML reports.
This is typically used prior ppp, to make sure the context is compatible with targetted accuracy. The generated report depends on the provided data. We only support observations from a single receiver. See --help
"
        )
        .long_about("TODO")
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
