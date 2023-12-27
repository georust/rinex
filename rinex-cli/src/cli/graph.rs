use clap::{value_parser, Arg, ArgAction, ArgMatches, ColorChoice, Command};

pub fn subcommand() -> Command {
    Command::new("graph")
        .short_flag('g')
        .long_flag("graph")
        .arg_required_else_help(true)
        .about(
            "RINEX dataset visualization (signals, orbits..), rendered as HTML in the workspace.",
        )
        .next_help_heading(
            "RINEX dependent visualizations. 
        Will only generate graphs if related dataset is present.",
        )
        .next_help_heading("GNSS combinations (requires OBS RINEX)")
        .arg(
            Arg::new("if")
                .short('i')
                .long("if")
                .action(ArgAction::SetTrue)
                .help("Plot Ionosphere Free (IF) signal combination."),
        )
        .arg(
            Arg::new("gf")
                .long("gf")
                .short('g')
                .action(ArgAction::SetTrue)
                .conflicts_with("no-graph")
                .help("Plot Geometry Free (GF) signal combination."),
        )
        .arg(
            Arg::new("wl")
                .long("wl")
                .short('w')
                .action(ArgAction::SetTrue)
                .help("Plot Wide Lane (WL) signal combination."),
        )
        .arg(
            Arg::new("nl")
                .long("nl")
                .short('n')
                .action(ArgAction::SetTrue)
                .conflicts_with("no-graph")
                .help("Plot Narrow Lane (WL) signal combination."),
        )
        .arg(
            Arg::new("mw")
                .long("mw")
                .short('m')
                .action(ArgAction::SetTrue)
                .conflicts_with("no-graph")
                .help("Plot Melbourne-Wübbena (MW) signal combination."),
        )
        .arg(
            Arg::new("dcb")
                .long("dcb")
                .short('d')
                .action(ArgAction::SetTrue)
                .conflicts_with("no-graph")
                .help("Differential Code Bias (DCB) visualization."),
        )
        .arg(
            Arg::new("multipath")
                .long("mp")
                .action(ArgAction::SetTrue)
                .conflicts_with("no-graph")
                .help("Code Multipath analysis"),
        )
        .arg(Arg::new("cs").long("cs").action(ArgAction::SetTrue).help(
            "Phase / Cycle Slip graph.
Plots raw phase signal with blackened sample where either CS was declared by receiver,
or we post processed determined a CS.",
        ))
        .next_help_heading("Clock states (requires: NAV RINEX, and/or CLK RINEX, and/or SP3)")
        .arg(
            Arg::new("sv-clock")
                .short('c')
                .long("clk")
                .action(ArgAction::SetTrue)
                .help("SV clock bias (offset, drift, drift changes)."),
        )
        .next_help_heading("Atmospheric Conditions")
        .arg(Arg::new("tec").long("tec").action(ArgAction::SetTrue).help(
            "Plot the TEC map. This is only feasible if at least one 
IONEX file is present in the context.",
        ))
}
