// filegen opmode
use clap::Command;

use super::{SHARED_DATA_ARGS, SHARED_GENERAL_ARGS};

pub fn subcommand() -> Command {
    Command::new("filegen")
        .long_flag("filegen")
        .arg_required_else_help(false)
        .about("Parse, preprocess and generate new data. See filegen --help")
        .long_about(
            "
`filegen` will synthesize new data after preprocessing.

For example, generate decimated Observation RINEX like this:

rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -P decim:5min \\
    filegen

`filegen` is a file operation (fops), any fops option may apply.
For example, redefine your production agency with:

rinex-cli \\
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \\
    -P decim:5min \\
    filegen -a CUSTOM
",
        )
        .next_help_heading("Production Environment")
        .args(SHARED_GENERAL_ARGS.iter())
        .next_help_heading("Data context")
        .args(SHARED_DATA_ARGS.iter())
}
