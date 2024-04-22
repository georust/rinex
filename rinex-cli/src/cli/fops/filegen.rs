// filegen opmode
use clap::Command;

use super::{SHARED_DATA_ARGS, SHARED_GENERAL_ARGS};

pub fn subcommand() -> Command {
    Command::new("filegen")
        .long_flag("filegen")
        .arg_required_else_help(false)
        .about(
            "Data generation opmode. 
Parse, process then generate data while preserving input format. 
You can use this mode to resample data, split it per constellation and much more..  
Refer to [https://github.com/georust/rinex/wiki/Preprocessing] for all processing algorithms.
",
        )
        .next_help_heading("Production Environment")
        .args(SHARED_GENERAL_ARGS.iter())
        .next_help_heading("Data context")
        .args(SHARED_DATA_ARGS.iter())
}
