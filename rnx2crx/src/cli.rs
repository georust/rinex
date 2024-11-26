use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};

use std::path::{Path, PathBuf};

pub struct Cli {
    /// arguments passed by user
    pub matches: ArgMatches,
}

impl Cli {
    pub fn new() -> Self {
        Self {
            matches: {
                Command::new("rnx2crx")
                    .author("Guillaume W. Bres <guillaume.bressaix@gmail.com>")
                    .version(env!("CARGO_PKG_VERSION"))
                    .about("RINEX compression tool")
                    .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .arg(
                        Arg::new("filepath")
                            .help("Input RINEX file")
                            .required(true),
                    )
                    .arg(
                        Arg::new("short")
                            .short('s')
                            .long("short")
                            .action(ArgAction::SetTrue)
                            .help("Prefer shortened filename convention.
Otherwise, we default to modern (V3+) long filenames.
Both will not work well if your input does not follow standard conventions at all."))
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .action(ArgAction::Set)
                            .help("Custom output file name. Otherwise, we follow standard conventions."))
                    .arg(
                        Arg::new("gz")
                            .long("gz")
                            .action(ArgAction::SetTrue)
                            .help("Force Gzip compression on the output.")
                    )
                    .arg(
                        Arg::new("workspace")
                            .short('w')
                            .action(ArgAction::Set)
                            .help("Define custom workspace.")
                    )
                    .get_matches()
            },
        }
    }
    pub fn input_path(&self) -> PathBuf {
        Path::new(self.matches.get_one::<String>("filepath").unwrap()).to_path_buf()
    }
    pub fn output_name(&self) -> Option<&String> {
        self.matches.get_one::<String>("output")
    }
    pub fn workspace(&self) -> Option<&String> {
        self.matches.get_one::<String>("workspace")
    }
    pub fn gzip_encoding(&self) -> bool {
        self.matches.get_flag("gz")
    }
}
