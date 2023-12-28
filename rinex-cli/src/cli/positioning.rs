// Positioning OPMODE
use clap::{Arg, ArgAction, Command};

pub fn subcommand() -> Command {
    Command::new("positioning")
        .short_flag('p')
        .arg_required_else_help(false)
        .about("Precise positioning opmode.
Use this mode to resolve precise positions and local time from RINEX dataset.
Expectes Observation RINEX from a single (unique) receiver.")
        .arg(Arg::new("cfg")
            .short('c')
            .long("cfg")
            .value_name("FILE")
            .required(false)
            .action(ArgAction::Append)
            .help("Pass a Position Solver configuration file (JSON).
[https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/struct.Config.html] is the structure to represent in JSON.
See [] for meaningful examples."))
        .arg(Arg::new("cggtts")
            .long("cggtts")
            .action(ArgAction::SetTrue)
            .help("Post processed PVT solutions wrapped in CGGTTS format for remote clock comparison (time transfer)."))
        .arg(Arg::new("gpx")
            .long("gpx")
            .action(ArgAction::SetTrue)
            .help("Format PVT solutions as GPX track."))
        .arg(Arg::new("kml")
            .long("kml")
            .action(ArgAction::SetTrue)
            .help("Format PVT solutions as KML track."))
}
