use clap::{
    Command,
    Arg, ArgMatches,
    ArgAction,
    ColorChoice,
};

use chrono::{
    NaiveDate, NaiveTime,
};

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
                    .version("1.0")
                    .about("RINEX compression tool")
                    .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .next_help_heading("Input/Output")
                    .arg(Arg::new("filepath")
                        .short('f')
                        .long("fp")
                        .help("Input RINEX file")
                        .required(true))
                    .arg(Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output RINEX file"))
                    .next_help_heading("Compression")
                    .arg(Arg::new("crx1")
                        .long("crx1")
                        .conflicts_with("crx3")
                        .action(ArgAction::SetTrue)
                        .help("Force to CRINEX1 compression"))
                    .arg(Arg::new("crx3")
                        .long("crx3")
                        .conflicts_with("crx1")
                        .action(ArgAction::SetTrue)
                        .help("Force to CRINEX3 compression"))
                    .arg(Arg::new("date")
                        .short('d')
                        .long("date")
                        .help("Set compression date, expects %Y-%m-%d description"))
                    .arg(Arg::new("time")
                        .short('t')
                        .long("time")
                        .help("Set compression time, expects %HH:%MM:%SS description"))
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
    pub fn crx1 (&self) -> bool {
        self.matches.get_flag("crx1")
    }
    pub fn crx3 (&self) -> bool {
        self.matches.get_flag("crx3")
    }
    pub fn date (&self) -> Option<NaiveDate> {
        if let Some(s) = self.matches
            .get_one::<String>("date") 
        {
            if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                Some(date)
            } else {
                println!("failed to parse provided date");
                None
            }
        } else {
            None
        }
    }
    pub fn time (&self) -> Option<NaiveTime> {
        if let Some(s) = self.matches
            .get_one::<String>("time") 
        {
            let items: Vec<&str> = s.split(":").collect();
            if items.len() != 3 {
                println!("failed to parse provided time");
                None
            } else {
                if let Ok(h) = u32::from_str_radix(items[0], 10) {
                    if let Ok(m) = u32::from_str_radix(items[1], 10) {
                        if let Ok(s) = u32::from_str_radix(items[2], 10) {
                            Some(NaiveTime::from_hms(h, m, s))
                        } else {
                            println!("failed to parse seconds");
                            None
                        }
                    } else {
                        println!("failed to parse minutes");
                        None
                    }
                } else {
                    println!("failed to parse hours");
                    None
                }
            }
        } else {
            None
        }
    }
}
