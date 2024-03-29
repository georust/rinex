// filegen opmode
use clap::Command;

use super::{SHARED_DATA_ARGS, SHARED_GENERAL_ARGS};

pub fn subcommand() -> Command {
    Command::new("filegen")
        .long_flag("filegen")
        .arg_required_else_help(false)
        .about(
            "RINEX Data formatting. Use this option to preprocess, 
modify and dump resulting context in preserved RINEX format. 
You can use this for example, to generate a decimated RINEX file from an input Observations file.",
        )
        .next_help_heading("Production Environment")
        .args(SHARED_GENERAL_ARGS.iter())
        .next_help_heading("Data context")
        .args(SHARED_DATA_ARGS.iter())
}
