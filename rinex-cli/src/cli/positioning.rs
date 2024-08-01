// Positioning OPMODE
use clap::{value_parser, Arg, ArgAction, Command};
use rinex::prelude::Duration;

fn shared_args(cmd: Command) -> Command {
    let cmd = cmd
        .arg(Arg::new("cfg")
            .short('c')
            .long("cfg")
            .value_name("FILE")
            .required(false)
            .action(ArgAction::Append)
            .help("Position Solver configuration file (JSON). See --help.")
            .long_help("
Read the [https://github.com/georust/rinex/wiki/Positioning] tutorial.
Use [https://github.com/georust/rinex/config] as a starting point.
[https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/struct.Config.html] is the structure to represent in JSON.
"));

    let cmd = if cfg!(feature = "kml") {
        cmd.arg(
            Arg::new("kml")
                .long("kml")
                .action(ArgAction::SetTrue)
                .help("Format PVT solutions as KML track."),
        )
    } else {
        cmd.arg(
            Arg::new("kml")
                .long("kml")
                .action(ArgAction::SetTrue)
                .help("[NOT AVAILABLE] requires kml compilation option"),
        )
    };

    let cmd = if cfg!(feature = "gpx") {
        cmd.arg(
            Arg::new("gpx")
                .long("gpx")
                .action(ArgAction::SetTrue)
                .help("Format PVT solutions as GPX track."),
        )
    } else {
        cmd.arg(
            Arg::new("gpx")
                .long("gpx")
                .action(ArgAction::SetTrue)
                .help("[NOT AVAILABLE] requires gpx compilation option"),
        )
    };

    let cmd =
        cmd.next_help_heading("CGGTTS (special resolution for clock comparison / time transfer)");

    if cfg!(not(feature = "cggtts")) {
        cmd.arg(
            Arg::new("cggtts")
                .long("cggtts")
                .action(ArgAction::SetTrue)
                .help("[NOT AVAILABLE] requires cggtts compilation option"),
        )
    } else {
        cmd
            .arg(Arg::new("cggtts")
                .long("cggtts")
                .action(ArgAction::SetTrue)
                .help("Activate CGGTTS special solver. See --help.")
                .long_help("Refer to the [https://github.com/georust/rinex/wiki/CGGTTS] tutorial."))
            .arg(Arg::new("tracking")
                .long("trk")
                .short('t')
                .value_parser(value_parser!(Duration))
                .action(ArgAction::Set)
                .help("CGGTTS custom tracking duration.
    Otherwise, the default tracking duration is used. Refer to [https://docs.rs/cggtts/latest/cggtts/track/struct.Scheduler.html]."))
            .arg(Arg::new("lab")
                .long("lab")
                .action(ArgAction::Set)
                .help("Define the name of your station or laboratory here."))
            .arg(Arg::new("utck")
                .long("utck")
                .action(ArgAction::Set)
                .conflicts_with("clock")
                .help("If the local clock tracks a local UTC replica, you can define the name
    of this replica here."))
            .arg(Arg::new("clock") 
                .long("clk")
                .action(ArgAction::Set)
                .conflicts_with("utck")
                .help("If the local clock is not a UTC replica and has a specific name, you
    can define it here."))
    }
}

pub fn ppp_subcommand() -> Command {
    let cmd = Command::new("ppp")
        .arg_required_else_help(false)
        .about(
            "Post Processed Positioning. Use this mode to deploy the precise position solver.
The solutions are added to the final report as an extra chapter. See --help",
        )
        .long_about(
            "Post Processed Positioning (ppp) opmode resolves
PVT solutions from RINEX data sampled by a single receiver (! This is not RTK!).
The solutions are presented in the analysis report (post processed results chapter).
Use --cggtts to convert solutions to CGGTTS special format.",
        );
    shared_args(cmd)
}

pub fn rtk_subcommand() -> Command {
    let cmd = Command::new("rtk")
        .arg_required_else_help(true)
        .about(
            "Post Processed RTK. Use this mode to deploy the precise differential positioning.
The initial context describes the Rover context. rtk accepts `-f` and `-d` once again, to describe the remote Station.
Other positioning flags still apply (like -c). See --help.",
        )
        .long_about(
            "RTK post opmode resolves PVT solutions by (post processed) differential navigation.
The initial context (-f, -d) describes the ROVER.
`rtk` also accepts -f and -d and you need to use those to describe the BASE (mandatory).
Other than that, `rtk` is stricly identical to `ppp` and is presented similarly.
CGGTTS and other options still apply."
        )
        .arg(
            Arg::new("fp")
                .long("fp")
                .value_name("FILE")
                .action(ArgAction::Append)
                .required_unless_present("dir")
                .help("Pass any RINEX file for remote base station"),
        )
        .arg(
            Arg::new("dir")
                .short('d')
                .value_name("DIR")
                .action(ArgAction::Append)
                .required_unless_present("fp")
                .help("Pass any directory for remote base station"),
        );
    shared_args(cmd)
}
