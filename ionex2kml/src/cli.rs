use clap::{
    Arg, //ArgAction,
    ArgMatches,
    ColorChoice,
    Command,
};
use log::error;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

use kml::types::KmlVersion;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("file type \"{0}\" is not supported")]
    FileTypeError(String),
    #[error("failed to parse ionex file")]
    IonexError(#[from] rinex::Error),
    #[error("failed to generate kml content")]
    GpxError(#[from] kml::Error),
}

#[derive(Debug, Clone, Default)]
pub struct Cli {
    matches: ArgMatches,
}

impl Cli {
    pub fn new() -> Self {
        Self {
            matches: {
                Command::new("ionex2kml")
                    .author("Guillaume W. Bres, <guillaume.bressaix@gmail.com>")
                    .version(env!("CARGO_PKG_VERSION"))
                    .about("IONEX to KML conveter")
                    .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .arg(
                        Arg::new("ionex")
                            .short('i')
                            .value_name("FILEPATH")
                            .help("Input IONEX file")
                            .required(true),
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .value_name("FILEPATH")
                            .help("Output KML file"),
                    )
                    //.arg(
                    //    Arg::new("equipotiential")
                    //        .short('n')
                    //        .value_name("N")
                    //        .help("Number of isosurfaces allowed"))
                    .next_help_heading("KML content")
                    .arg(
                        Arg::new("version")
                            .short('v')
                            .value_name("VERSION")
                            .help("Define specific KML Revision"),
                    )
                    .arg(
                        Arg::new("attributes")
                            .short('a')
                            .value_name("[NAME,VALUE]")
                            .help("Define custom file attributes"),
                    )
                    .get_matches()
            },
        }
    }
    /// Returns KML version to use, based on user command line
    pub fn kml_version(&self) -> KmlVersion {
        if let Some(version) = self.matches.get_one::<String>("version") {
            if let Ok(version) = KmlVersion::from_str(version) {
                version
            } else {
                let default = KmlVersion::default();
                error!("invalid KML version, using default value \"{:?}\"", default);
                default
            }
        } else {
            KmlVersion::default()
        }
    }
    /// Returns optional <String, String> "KML attributes"
    pub fn kml_attributes(&self) -> Option<HashMap<String, String>> {
        if let Some(attributes) = self.matches.get_one::<String>("attributes") {
            let content: Vec<&str> = attributes.split(",").collect();
            if content.len() > 1 {
                Some(
                    vec![(content[0].to_string(), content[1].to_string())]
                        .into_iter()
                        .collect(),
                )
            } else {
                error!("invalid csv, need a \"field\",\"value\" description");
                None
            }
        } else {
            None
        }
    }
    // /// Returns nb of equipotential TEC map we will exhibit,
    // /// the maximal error on resulting TEC is defined as max|tec_u| / n
    // pub fn nb_tec_potentials(&self) -> usize {
    //     //if let Some(n) = self.matches.get_one::<u32>("equipoential") {
    //     //    *n as usize
    //     //} else {
    //         20 // default value
    //     //}
    // }
    /// Returns ionex filename
    pub fn ionex_filepath(&self) -> &str {
        self.matches.get_one::<String>("ionex").unwrap()
    }
}
