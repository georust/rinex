// tbin opmode
use clap::{Arg, ArgAction, Command};

pub fn subcommand() -> Command {
    Command::new("quality-check")
        .short_flag('Q')
        .long_flag("qc")
        .about(
            "File Quality analysis (statistical evaluation) of the dataset.
Typically used prior precise point positioning.",
        )
        .arg(
            Arg::new("spp")
                .long("spp")
                .action(ArgAction::SetTrue)
                .help("Force solving method to SPP.
Otherwise we use the default Method.
See online documentations [https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html#variants]."))
        .arg(
            Arg::new("cfg")
                .short('c')
                .long("cfg")
                .required(false)
                .value_name("FILE")
                .action(ArgAction::Append)
                .help("Pass a QC configuration file (JSON).
[] is the structure to represent in JSON.
See [] for meaningful examples."))
}
