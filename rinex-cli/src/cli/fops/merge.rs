// Merge opmode
use clap::{value_parser, Arg, ArgAction, Command};
use std::path::PathBuf;

use super::{SHARED_DATA_ARGS, SHARED_GENERAL_ARGS};

pub fn subcommand() -> Command {
    Command::new("merge")
        .short_flag('m')
        .long_flag("merge")
        .arg_required_else_help(true)
        .about("Merge a RINEX into another and dump result. See -m --help.")
        .long_about(
            "Merge two files together.

1. OBS RINEX example.
When working with OBS RINEX, you should consider files that come from the same station.
Merge GPS data content, from two files into a single RINEX

rinex-cli \\
   -f test_resources/OBS/V3/VLNS0010.22O \\
   -P GPS \\
   -m test_resources/OBS/V3/VLNS0630.22O

1b. CRINEX example.
When working with CRINEX, the file format is preserved:

rinex-cli \\
  -f test_resources/CRNX/V3/ACOR00ESP_R_20213550000_01D_30S_MO.crx \\
  -m test_resources/CRNX/V3/BME100HUN_R_20213550000_01D_30S_MO.crx

2. Compressed file.
File compression is also preserved:

rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -m test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz
   ",
        )
        .arg(
            Arg::new("file")
                .value_parser(value_parser!(PathBuf))
                .value_name("FILEPATH")
                .action(ArgAction::Set)
                .required(true)
                .help("RINEX file to merge."),
        )
        .next_help_heading("Production Environment")
        .args(SHARED_GENERAL_ARGS.iter())
        .next_help_heading("Data context")
        .args(SHARED_DATA_ARGS.iter())
}
