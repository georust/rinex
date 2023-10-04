use std::env;
use std::io::Write;
use std::path::Path;

fn build_nav_database() {
    let outdir = env::var("OUT_DIR").unwrap();
    let nav_path = Path::new(&outdir).join("nav_orbits.rs");
    let mut nav_file = std::fs::File::create(&nav_path).unwrap();

    // read helper descriptor
    let nav_descriptor = std::fs::read_to_string("db/NAV/orbits.json").unwrap();
    // parse
    let json: serde_json::Value = serde_json::from_str(&nav_descriptor).unwrap();

    let nav_frames = json.as_array().unwrap();

    let nav_content = "use lazy_static::lazy_static;
use crate::version::Version;
use crate::prelude::Constellation;
use crate::navigation::NavMsgType;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[derive(Eq, Ord)]
pub struct NavHelper<'a> {
    pub constellation: Constellation,
    pub version: Version,
    pub msg: NavMsgType,
    pub items: Vec<(&'a str, &'a str)>,
}

// NAV FRAMES description, from RINEX standards
";
    nav_file.write_all(nav_content.as_bytes()).unwrap();

    nav_file.write_all("lazy_static! {\n".as_bytes()).unwrap();

    nav_file.write_all("#[derive(Debug)]\n".as_bytes()).unwrap();

    nav_file
        .write_all("   pub static ref NAV_ORBITS: Vec<NavHelper<'static>> = vec![ \n".as_bytes())
        .unwrap();

    for frame in nav_frames {
        // grab all fields
        let constellation = frame["constellation"].as_str().unwrap(); // mandatory

        let major = frame["version"]["major"].as_u64().unwrap(); // major is mandatory

        let minor = frame["version"]["minor"].as_u64().unwrap_or(0); // optionnal

        let msg = frame["type"].as_str().unwrap_or("LNAV"); // default type

        let items = frame["orbits"].as_object().unwrap(); // mandatory

        // begin frame description
        nav_file.write_all("   ( NavHelper {\n".as_bytes()).unwrap();
        nav_file
            .write_all(
                format!(
                    "      constellation: Constellation::from_str(\"{}\").unwrap(),\n",
                    constellation
                )
                .as_bytes(),
            )
            .unwrap();
        nav_file
            .write_all("      version: Version {\n".as_bytes())
            .unwrap();
        nav_file
            .write_all(format!("           major: {},\n", major).as_bytes())
            .unwrap();
        nav_file
            .write_all(format!("           minor: {},\n", minor).as_bytes())
            .unwrap();
        nav_file.write_all("      },\n".as_bytes()).unwrap();
        nav_file
            .write_all(
                format!("      msg: NavMsgType::from_str(\"{}\").unwrap(),\n", msg).as_bytes(),
            )
            .unwrap();
        // frame body description
        nav_file
            .write_all("      items: vec![ \n".as_bytes())
            .unwrap();
        for key in items.keys() {
            nav_file
                .write_all(format!("         (\"{}\",{}),\n", key, items[key]).as_bytes())
                .unwrap();
        }
        nav_file.write_all("      ],\n".as_bytes()).unwrap();
        nav_file.write_all("   }),\n".as_bytes()).unwrap();
    }

    nav_file
        .write_all("   ];\n".as_bytes()) // NAV_ORBITS vec![
        .unwrap();

    nav_file
        .write_all("}\n".as_bytes()) // lazy_static!
        .unwrap();
}

use serde::Deserialize;

fn default_launch_month() -> u8 {
    1 // Jan
}

fn default_launch_day() -> u8 {
    1 // 1st day of month
}

/*
 * We use an intermediate struct
 * and "serde" to allow not to describe the launched
 * day or month for example
 */
#[derive(Deserialize)]
struct SBASDBEntry<'a> {
    pub constellation: &'a str,
    pub prn: u16,
    pub id: &'a str,
    #[serde(default = "default_launch_month")]
    pub launched_month: u8,
    #[serde(default = "default_launch_day")]
    pub launched_day: u8,
    pub launched_year: i32,
}

fn build_sbas_helper() {
    let outdir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&outdir).join("sbas.rs");
    let mut fd = std::fs::File::create(path).unwrap();

    // read descriptor: parse and dump into a static array
    let db_content = std::fs::read_to_string("db/SBAS/sbas.json").unwrap();

    let sbas_db: Vec<SBASDBEntry> = serde_json::from_str(&db_content).unwrap();

    let content = "use lazy_static::lazy_static;

#[derive(Debug)]
pub struct SBASHelper<'a> {
    constellation: &'a str,
    prn: u16,
    id: &'a str,
    launched_day: u8,
    launched_month: u8,
    launched_year: i32,
}

lazy_static! {
    static ref SBAS_VEHICLES: Vec<SBASHelper<'static>> = vec![
\n";

    fd.write_all(content.as_bytes()).unwrap();

    for e in sbas_db {
        fd.write_all(
            format!(
                "SBASHelper {{
                constellation: \"{}\",
                prn: {},
                id: \"{}\",
                launched_year: {},
                launched_month: {},
                launched_day: {}
            }},",
                e.constellation, e.prn, e.id, e.launched_year, e.launched_month, e.launched_day,
            )
            .as_bytes(),
        )
        .unwrap()
    }

    fd.write_all("    ];".as_bytes()).unwrap();
    fd.write_all("}\n".as_bytes()).unwrap();
}

fn main() {
    build_nav_database();
    if cfg!(feature = "sbas") {
        build_sbas_helper();
    }
}
