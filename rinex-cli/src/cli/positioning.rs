// Positioning OPMODE
use clap::{value_parser, Arg, ArgAction, Command};
use rinex::prelude::Duration;

pub fn subcommand() -> Command {
    Command::new("positioning")
        .short_flag('p')
        .arg_required_else_help(false)
        .about("Precise Positioning opmode.
Use this mode to resolve Position Velocity and Time (PVT) solutions from one GNSS context.")
        .arg(Arg::new("cfg")
            .short('c')
            .long("cfg")
            .value_name("FILE")
            .required(false)
            .action(ArgAction::Append)
            .help("Pass a Position Solver configuration file (JSON). See --help.")
            .long_help("
Use [https://github.com/georust/rinex/rinex-cli/config.rtk] as a starting point.
[https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/struct.Config.html] is the structure to represent in JSON.
Our Wiki pages contains several examples."))
        .arg(Arg::new("gpx")
            .long("gpx")
            .action(ArgAction::SetTrue)
            .help("Format PVT solutions as GPX track."))
        .arg(Arg::new("kml")
            .long("kml")
            .action(ArgAction::SetTrue)
            .help("Format PVT solutions as KML track."))
        .next_help_heading("CGGTTS (special resolution for clock comparison / time transfer)")
        .arg(Arg::new("cggtts")
            .long("cggtts")
            .action(ArgAction::SetTrue)
            .help("Activate CGGTTS special solver. See --help.")
            .long_help("In CGGTTS opmode, we're only interested in resolving the local offset to the constellation.
Navigation mode is set to [TimeOnly] and we navigate using every single vehicle in sight fitting criteria. 
CGGTTS opmode is therefore more demanding as it runs the algorithm many more times than regular PPP.
The PVT solutions are then formatted as a CGGTTS file which is used to compare remote clocks to one another, from a common GNSS constellation."))
        .arg(Arg::new("tracking")
            .long("trk")
            .short('t')
            .value_parser(value_parser!(Duration))
            .action(ArgAction::Set)
            .help("CGGTTS custom tracking duration.
Otherwise, the default tracking duration is used.
Refer to []"))
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
