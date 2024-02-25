// filegen opmode
use clap::{value_parser, Arg, Command};

pub fn subcommand() -> Command {
    Command::new("filegen")
        .long_flag("filegen")
        .arg_required_else_help(false)
        .about(
            "RINEX Data formatting. Use this option to preprocess, 
modify and dump resulting context in preserved RINEX format. 
You can use this for example, to generate a decimated RINEX file from an input Observations file.",
        )
}
