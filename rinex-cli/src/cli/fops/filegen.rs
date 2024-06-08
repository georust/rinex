// filegen opmode
use clap::Command;

use super::{SHARED_DATA_ARGS, SHARED_GENERAL_ARGS};

pub fn subcommand() -> Command {
    Command::new("filegen")
        .long_flag("filegen")
        .arg_required_else_help(false)
        .about(
            "Parse, preprocess and generate data while preserving input format. See --filegen --help."
        )
        .long_about("
Use this mode to generate all file formats we support after preprocessing them.

Example (1): generate decimated RINEX Observations
rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -P decim:5min \\
    --filegen

Example (2): redefine production agency while we do that
rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -P decim:5min \\
    --filegen -a AGENCY
"
        )
        .next_help_heading("Production Environment")
        .args(SHARED_GENERAL_ARGS.iter())
        .next_help_heading("Data context")
        .args(SHARED_DATA_ARGS.iter())
}
