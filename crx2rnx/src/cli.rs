use clap::{
    Command,
    Arg, ArgMatches,
    ColorChoice,
};

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
                    .arg(Arg::new("filepath")
                        .short('f')
                        .long("fp")
                        .help("Input RINEX file")
                        .required(true))
                    .arg(Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output RINEX file"))
                    .get_matches()
            }
        }
    }
    pub fn input_path(&self) -> &str {
        &self.matches
            .get_one::<String>("filepath")
            .unwrap()
    }
    pub fn output_path(&self) -> Option<&String> {
        self.matches
            .get_one::<String>("output")
    }
}
