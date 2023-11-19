use clap::{Arg, ArgMatches, ColorChoice, Command};
use std::path::{Path, PathBuf};

pub struct Cli {
    /// arguments passed by user
    pub matches: ArgMatches,
}

impl Cli {
    pub fn new() -> Self {
        Self {
            matches: {
                Command::new("crx2rnx")
                    .author("Guillaume W. Bres <guillaume.bressaix@gmail.com>")
                    .version("2.0")
                    .about("Compact RINEX decompression tool")
                    .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .arg(
                        Arg::new("filepath")
                            .short('f')
                            .long("fp")
                            .help("Input RINEX file")
                            .required(true),
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .help("Custom output file name"),
                    )
                    .arg(
                        Arg::new("workspace")
                            .short('w')
                            .long("workspace")
                            .help("Define custom workspace location"),
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
}
