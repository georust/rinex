pub mod diff;
pub mod filegen;
pub mod merge;
pub mod split;
pub mod time_binning;

use lazy_static::lazy_static;

use ::clap::{value_parser, Arg, ArgAction};

use rinex::prod::{DataSource, FFU, PPU};

/*
 * Arguments that are shared by all file operations.
 * Mainly [ProductionAttributes] (re)definition opts
 */
lazy_static! {
    pub static ref SHARED_GENERAL_ARGS : Vec<Arg> = vec![
        Arg::new("batch")
            .short('b')
            .long("batch")
            .required(false)
            .value_parser(value_parser!(u8))
            .help("Set # (number ID) in case this file is part of a file serie"),
        Arg::new("short")
            .short('s')
            .long("short")
            .action(ArgAction::SetTrue)
            .help("Prefer (deprecated) short filenames as historically used.
Otherwise, this ecosystem prefers modern (longer) filenames that contain more information."),
        Arg::new("gzip")
            .long("gzip")
            .action(ArgAction::SetTrue)
            .help("Force .gzip compressed file generation, even if input data is not."),
        Arg::new("unzip")
            .long("unzip")
            .action(ArgAction::SetTrue)
            .help("Force plain/readable file generation. By default, if input data is gzip compressed, we will preserve
the input compression. Use this to bypass."),
        Arg::new("csv")
            .long("csv")
            .action(ArgAction::SetTrue)
            .help("Extract dataset and generate as CSV instead of RINEX/SP3.
Use this when targetting third party tools."),
        Arg::new("agency")
            .short('a')
            .long("agency")
            .required(false)
            .help("Define a custom agency name, possibly overwriting
what the original filename did define (according to conventions)."),
        Arg::new("country")
            .short('c')
            .long("country")
            .required(false)
            .help("Define a custom (3 letter) country code.
This code should represent where the Agency is located."),
        Arg::new("source")
            .long("src")
            .required(false)
            .value_name("[RCVR,STREAM]")
            .value_parser(value_parser!(DataSource))
            .help("Define the data source.
In RINEX standards, we use \"RCVR\" when data was sampled from a hardware receiver.
Use \"STREAM\" for other stream data source, like RTCM for example.")
    ];

    pub static ref SHARED_DATA_ARGS : Vec<Arg> = vec![
        Arg::new("PPU")
            .long("ppu")
            .required(false)
            .value_name("[15M,01H,01D,01Y]")
            .value_parser(value_parser!(PPU))
            .help("Define custom production periodicity (time between two batch/dataset).
\"15M\": 15' interval, \"01H\": 1 hr interval, \"01D\": 1 day interval, \"01Y\": 1 year interval"),
        Arg::new("FFU")
            .long("ffu")
            .required(false)
            .value_name("DDU")
            .value_parser(value_parser!(FFU))
            .help("Define custom sampling interval.
 Note that this only affects the filename to be generated, inner Record should match for consistency.
 The sampling interval is the dominant time delta between two Epoch inside the record.
 Format is \"DDU\" where DD must be (at most) two valid digits and U is the time Unit.
 For example: \"30S\" for 30sec. interval, \"90S\" for 1'30s interval, \"20M\" for 20' interval, \"02H\" for 2hr interval, \"07D\" for weekly interval."),
        Arg::new("region")
            .long("region")
            .value_name("[G]")
            .help("Regional code, solely used in IONEX file name.
Use this to accurately (re)define your IONEX context that possibly did not follow standard naming conventions.
Use `G` for Global (World wide) TEC maps."),
    ];
}
