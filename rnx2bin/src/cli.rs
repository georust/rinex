use binex::prelude::Meta;
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
                Command::new("rnx2bin")
                    .author("Guillaume W. Bres <guillaume.bressaix@gmail.com>")
                    .version(env!("CARGO_PKG_VERSION"))
                    .about("RINEX to BINEX")
                    .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .arg(
                        Arg::new("filepath")
                            .help("Input RINEX file")
                            .required(true),
                    )
                    .arg(
                        Arg::new("workspace")
                            .short('w')
                            .action(ArgAction::Set)
                            .help("Define custom workspace.")
                    )
                    .next_help_heading("BINEX (forging)")
                    .arg(
                        Arg::new("little")
                            .short('l')
                            .long("little")
                            .action(ArgAction::SetTrue)
                            .help("Encode using Little endianness, otherwise Big endianness is prefered.")
                    )
                    .arg(
                        Arg::new("crc")
                            .long("crc")
                            .action(ArgAction::SetTrue)
                            .help("Encode using enhanced CRC scheme (very robust messaging).")
                        )
                    .arg(
                        Arg::new("reversed")
                            .short('r')
                            .action(ArgAction::SetTrue)
                            .help("Forge a Reversed BINEX Stream.")
                    )
                    .next_help_heading("Output")
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .action(ArgAction::Set)
                            .help("Custom output. When omitted, we will auto-generate a bin file.
This can be either
- a file name that we will generate
- or a streaming interface, for example /dev/fifo0 that is writable and accepts raw bytes.
In this case, the --io flag must be specified as well.
either be a custom file name (example: output.bin), otherwise, standard convention is auto generated.")
                    )
                    .arg(
                        Arg::new("io")
                            .long("io")
                            .action(ArgAction::SetTrue)
                            .help("Custom output name (example: output.bin), otherwise, standard convention is auto generated.")
                    )
                    .arg(
                        Arg::new("short")
                            .short('s')
                            .long("short")
                            .action(ArgAction::SetTrue)
                            .help("Specify that the auto file name generator should prefer short (V2) file names")
                    )
                    .arg(
                        Arg::new("gzip")
                            .long("gzip")
                            .action(ArgAction::SetTrue)
                            .help("Force gzip output, even coming from uncompressed input.
This works on any type of output interface.")
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
    pub fn gzip_output(&self) -> bool {
        self.matches.get_flag("gzip")
    }
    pub fn io_output(&self) -> bool {
        self.matches.get_flag("io")
    }
    pub fn short_name(&self) -> bool {
        self.matches.get_flag("short")
    }
    pub fn meta(&self) -> Meta {
        Meta {
            reversed: self.matches.get_flag("reversed"),
            enhanced_crc: self.matches.get_flag("crc"),
            big_endian: !self.matches.get_flag("little"),
        }
    }
}
