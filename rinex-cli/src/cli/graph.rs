use clap::{Arg, ArgAction, Command};

pub fn subcommand() -> Command {
    Command::new("graph")
        .short_flag('g')
        .long_flag("graph")
        .arg_required_else_help(true)
        .about(
            "RINEX data analysis and visualization, rendered as HTML or CSV in the workspace. See -g --help.",
        )
        .long_about("Analysis and plots (in HTML).
When Observations are present, whether they come from Observation RINEX, Meteo or DORIS RINEX,
we can export the results as CSV too. This is particularly useful to export the results of the analysis
to other tools.")
        .arg(
            Arg::new("csv")
                .long("csv")
                .action(ArgAction::SetTrue)
                .help("Extract Data as CSV along HTML plots. See --help.")
                .long_help("This is particularly helpful if you are interested in
using our toolbox as data parser and preprocessor and inject the results to third party programs.")                
        )
        .next_help_heading(
            "RINEX dependent visualizations. 
        Will only generate graphs if related dataset is present.",
        )
        .next_help_heading("Observations rendering (OBS, Meteo, DORIS)")
        .arg(
            Arg::new("obs")
                .short('o')
                .long("obs")
                .action(ArgAction::SetTrue)
                .help("Plot all observables described in either Observation, Meteo or DORIS RINEX. See --help")
                .long_help("Use this option to plot all observations.
OBS RINEX gives GNSS signals observations, but we also support Meteo RINEX and DORIS (special observation) RINEX.

Example (1): render GNSS signals (all of them, whether it be Phase or PR) for GPS.
Extract as CSV at the same time:

./target/release/rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -P GPS -g --obs --csv

Example (2): render meteo sensor observations similary.

./target/release/rinex-cli \\
    -f test_resources/MET/V3/POTS00DEU_R_20232540000_01D_05M_MM.rnx.gz \\
    -g --obs --csv

Example (3): render DORIS observations similarly.

./target/release/rinex-cli \\
    -f test_resources/OR/V3/cs2rx18164.gz -g --obs --csv

Example (4): render OBS + Meteo combination at once.
RINEX-Cli allows loading OBS and Meteo in one session.
In graph mode, this means we can render both in a single run.

./target/release/rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -f test_resources/MET/V3/POTS00DEU_R_20232540000_01D_05M_MM.rnx.gz \\
    -g --obs --csv
")

        )
        .next_help_heading("GNSS signals (requires OBS and/or DORIS RINEX)")
        .arg(
            Arg::new("dcb")
                .long("dcb")
                .action(ArgAction::SetTrue)
                .help("Plot Differential Code Bias.")
                .long_help(
"Plot Differential Code bias of the 5 following spacecrafts

./target/release/rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -P G06,E13,C14,G15,E31 \\
    -g --dcb")
        )
        .arg(
            Arg::new("mp")
                .long("mp")
                .action(ArgAction::SetTrue)
                .help("Plot Code Multipath.")
                .long_help(
"Plot Code Multipath bias from the 5 following spacecrafts

./target/release/rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -P G06,E13,C14,G15,E31 \\
    -g --mp")
        )
        .arg(
            Arg::new("if")
                .short('i')
                .long("if")
                .action(ArgAction::SetTrue)
                .help("Plot Ionosphere Free (IF) signal combination.")
                .long_help(
"Plot Ionosphere free signal combination, for the 5 following spacecrafts

./target/release/rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -P G06,E13,C14,G15,E31 \\
    -g --if")
        )
        .arg(
            Arg::new("gf")
                .long("gf")
                .short('g')
                .action(ArgAction::SetTrue)
                .help("Plot Geometry Free (GF) signal combination.")
                .long_help(
"Plot Geometry free signal combination, for the 5 following spacecrafts

./target/release/rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -P G06,E13,C14,G15,E31 \\
    -g --gf")
        )
        .arg(
            Arg::new("wl")
                .long("wl")
                .short('w')
                .action(ArgAction::SetTrue)
                .help("Plot Wide Lane (WL) signal combination.")
                .long_help(
"Plot Widelane signal combination, for the 5 following spacecrafts

./target/release/rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -P G06,E13,C14,G15,E31 \\
    -g --wl")
        )
        .arg(
            Arg::new("nl")
                .long("nl")
                .short('n')
                .action(ArgAction::SetTrue)
                .help("Plot Narrow Lane (WL) signal combination.")
                .long_help(
"Plot Narrowlane signal combination, for the 5 following spacecrafts

./target/release/rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -P G06,E13,C14,G15,E31 \\
    -g --nl")
        )
        .arg(
            Arg::new("mw")
                .long("mw")
                .short('m')
                .action(ArgAction::SetTrue)
                .help("Plot Melbourne-Wübbena (MW) signal combination.")
                .long_help(
"Plot Melbourne-Wubbena signal combination for the 5 following spacecrafts

./target/release/rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -P G06,E13,C14,G15,E31 \\
    -g --mw")
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
                .help("3D projection of SV attitudes in the sky."),
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
                .help("Plot the TEC map. Requires at least one IONEX file. See --help")
                .long_help("Plot the worldwide TEC map, usually presented in 24hr time frame. 
Example:
rinex-cli -f test_resources/IONEX/V1/CKMG0080.09I.gz -g --tec")
        )
        .arg(
            Arg::new("ionod")
                .long("ionod")
                .action(ArgAction::SetTrue)
                .help("Plot ionospheric delay per signal & SV, at latitude and longitude of signal sampling."),
        )
}
