use clap::{Arg, ArgAction, Command};

pub fn subcommand() -> Command {
    Command::new("graph")
        .short_flag('g')
        .long_flag("graph")
        .arg_required_else_help(true)
        .about(
            "RINEX data visualization (signals, orbits..), rendered as HTML or CSV in the workspace.",
        )
        .arg(
            Arg::new("csv")
                .long("csv")
                .action(ArgAction::SetTrue)
                .help("Generate CSV files along HTML plots.")
        )
        .next_help_heading(
            "RINEX dependent visualizations. 
        Will only generate graphs if related dataset is present.",
        )
        .next_help_heading("GNSS observations (requires OBS RINEX)")
        .arg(
            Arg::new("obs")
                .short('o')
                .long("obs")
                .action(ArgAction::SetTrue)
                .help(
                    "Plot all observables.
When OBS RINEX is provided, this will plot raw phase, dopplers and SSI.
When METEO RINEX is provided, data from meteo sensors is plotted too.",
                ),
        )
        .arg(
            Arg::new("dcb")
                .long("dcb")
                .action(ArgAction::SetTrue)
                .help("Plot Differential Code Bias. Requires OBS RINEX."),
        )
        .arg(
            Arg::new("mp")
                .long("mp")
                .action(ArgAction::SetTrue)
                .help("Plot Code Multipath. Requires OBS RINEX."),
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
        .arg(Arg::new("cs").long("cs").action(ArgAction::SetTrue).help(
            "Phase / Cycle Slip graph.
Plots raw phase signal with blackened sample where either CS was declared by receiver,
or we post processed determined a CS.",
        ))
        .next_help_heading("Navigation (requires NAV RINEX and/or SP3)")
        .arg(
            Arg::new("skyplot")
                .short('s')
                .long("sky")
                .action(ArgAction::SetTrue)
                .help("Skyplot: SV position in the sky, on a compass."),
        )
        .arg(
            Arg::new("orbit")
                .long("orbit")
                .action(ArgAction::SetTrue)
                .help("SV position in the sky, on 2D cartesian plots."),
        )
        .arg(
            Arg::new("orbit-residual")
                .long("orbit-residual")
                .action(ArgAction::SetTrue)
                .help(
                    "Broadcast versus High Precision orbital product comparison |BRDC - SP3|.
Requires both NAV RINEX and SP3 that overlap in time.
It is the orbital equuivalent to |BRDC-CLK| requested with --clk-residual."))
        .arg(
            Arg::new("naviplot")
                .long("naviplot")
                .action(ArgAction::SetTrue)
                .help(
                    "SV orbital attitude projected in 3D.
Ideal for precise positioning decision making.",
                ),
        )
        .next_help_heading("Clock states (requires either NAV RINEX, CLK RINEX or SP3)")
        .arg(
            Arg::new("sv-clock")
                .short('c')
                .long("clk")
                .action(ArgAction::SetTrue)
                .help("SV clock bias (offset, drift, drift changes).")
        )
        .arg(
            Arg::new("clk-residual")
                .long("clk-residual")
                .action(ArgAction::SetTrue)
                .help("Broadcast versus High Precision clock product comparison |BRDC - CLK|.
Requires both NAV RINEX and Clock RINEX that overlap in time.
It is the temporal equuivalent to |BRDC-SP3| requested with --sp3-residual.")
        )
        .next_help_heading("Atmosphere conditions")
        .arg(
            Arg::new("tropo")
                .long("tropo")
                .action(ArgAction::SetTrue)
                .help("Plot tropospheric delay from meteo sensors estimation. Requires METEO RINEX."),
        )
        .arg(
            Arg::new("tec")
                .long("tec")
                .action(ArgAction::SetTrue)
                .help("Plot the TEC map. Requires at least one IONEX file."),
        )
        .arg(
            Arg::new("ionod")
                .long("ionod")
                .action(ArgAction::SetTrue)
                .help("Plot ionospheric delay per signal & SV, at latitude and longitude of signal sampling."),
        )
}
