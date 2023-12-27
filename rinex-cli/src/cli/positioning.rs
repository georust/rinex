// Positioning OPMODE
use clap::Command;

fn subcommand() -> Command {
    Command::new("positioning")
        .short_flag("p")
        .arg_required_else_help(true)
        .help("Post processed Positioning opmode.
        Use this mode to resolve precise positions and local time, from
        provided RINEX dataset.")
        .arg(Arg::new("config")
            .short('c')
            .value_name("FILE")
            .help("Pass a Position Solver configuration file (JSON).
        [https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/struct.Config.html] is the structure to represent. Format is JSON. 
        See [] for meaningful examples."))
}
