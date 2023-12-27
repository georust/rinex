// Data identification opmode
use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};

pub fn subcommand() -> Command {
    Command::new("identify")
        .short_flag('i')
        .long_flag("id")
        .arg_required_else_help(true)
        .about("RINEX data identification opmode")
        .arg(
            Arg::new("all").short('a').action(ArgAction::SetTrue).help(
                "Complete RINEX dataset(s) identification. Turns on all following algorithms.",
            ),
        )
        .arg(
            Arg::new("epochs")
                .long("epochs")
                .short('e')
                .action(ArgAction::SetTrue)
                .help("Epoch, Time system and sampling analysis."),
        )
        .arg(
            Arg::new("gnss")
                .long("gnss")
                .short('g')
                .action(ArgAction::SetTrue)
                .help("Enumerate GNSS constellations."),
        )
        .arg(
            Arg::new("sv")
                .long("sv")
                .short('s')
                .action(ArgAction::SetTrue)
                .help("Enumerates SV."),
        )
        .arg(
            Arg::new("header")
                .long("header")
                .short('h')
                .action(ArgAction::SetTrue)
                .help("Extracts major header fields"),
        )
        .next_help_heading(
            "Following sections are RINEX specific. They will only apply to the related subset.",
        )
        .next_help_heading("Observation RINEX")
        .arg(
            Arg::new("observables")
                .long("obs")
                .short('o')
                .action(ArgAction::SetTrue)
                .help("Identify observables in either Observation or Meteo dataset(s)."),
        )
        .arg(
            Arg::new("snr")
                .long("snr")
                .action(ArgAction::SetTrue)
                .help("SNR identification ([min, max] range, per SV..)"),
        )
        .arg(
            Arg::new("anomalies")
                .short('a')
                .long("anomalies")
                .action(ArgAction::SetTrue)
                .help(
                    "Enumerate abnormal events along the input time frame (epochs).
           Abnormal events are unexpected receiver reset or possible cycle slips for example.",
                ),
        )
        .next_help_heading("Navigation (BRDC) RINEX")
        .arg(
            Arg::new("nav-msg")
                .long("nav-msg")
                .action(ArgAction::SetTrue)
                .help("Identify Navigation frame types."),
        )
}
