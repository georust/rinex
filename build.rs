use std::env;
use std::io::Write;
use std::path::Path;

fn build_nav_database() {
    let outdir = env::var("OUT_DIR").unwrap();
    let nav_path = Path::new(&outdir).join("nav_orbits.rs");
    let mut nav_file = std::fs::File::create(nav_path).unwrap();

    // read helper descriptor
    let nav_descriptor = std::fs::read_to_string("db/NAV/orbits.json").unwrap();
    // parse
    let json: serde_json::Value = serde_json::from_str(&nav_descriptor).unwrap();

    let nav_frames = json.as_array().unwrap();

    let nav_content = "use lazy_static::lazy_static;

use crate::{
    prelude::{Version, Constellation},
    navigation::NavMessageType,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[derive(Eq, Ord)]
pub struct NavHelper<'a> {
    pub constellation: Constellation,
    pub version: Version,
    pub msg: NavMessageType,
    pub items: Vec<(&'a str, &'a str)>,
    pub row_lengths: Vec<usize>,
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

        // Older descriptors use four fields per row, with a possibly shorter
        // final row. Explicit lengths describe records such as Galileo NAV4.
        let row_lengths: Vec<usize> = match frame.get("row_lengths") {
            Some(lengths) => lengths
                .as_array()
                .expect("row_lengths must be an array")
                .iter()
                .map(|length| {
                    usize::try_from(length.as_u64().expect("row length must be an integer"))
                        .expect("row length must fit usize")
                })
                .collect(),
            None => (0..items.len())
                .step_by(4)
                .map(|start| (items.len() - start).min(4))
                .collect(),
        };
        assert!(
            row_lengths.iter().all(|length| (1..=4).contains(length)),
            "{constellation} {major}.{minor:02} {msg}: row lengths must be 1..=4"
        );
        assert_eq!(
            row_lengths.iter().sum::<usize>(),
            items.len(),
            "{constellation} {major}.{minor:02} {msg}: row lengths must cover all fields"
        );

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
                format!(
                    "      msg: NavMessageType::from_str(\"{}\").unwrap(),\n",
                    msg
                )
                .as_bytes(),
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
        nav_file
            .write_all(format!("      row_lengths: vec!{:?},\n", row_lengths).as_bytes())
            .unwrap();
        nav_file.write_all("   }),\n".as_bytes()).unwrap();
    }

    nav_file
        .write_all("   ];\n".as_bytes()) // NAV_ORBITS vec![
        .unwrap();

    nav_file
        .write_all("}\n".as_bytes()) // lazy_static!
        .unwrap();
}

fn main() {
    build_nav_database();
}
