use std::env;
use std::io::Write;
use std::path::Path;

fn build_nav_database() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let nav_path = Path::new(&out_dir).join("nav_orbits.rs");
    let mut nav_file = std::fs::File::create(&nav_path).unwrap();
    let nav_data = std::fs::read_to_string("db/NAV/orbits.json").unwrap();
    let json: serde_json::Value = serde_json::from_str(&nav_data).unwrap();
    let constellations = json.as_array().unwrap();
    nav_file
        .write_all("use lazy_static::lazy_static;\n\n".as_bytes())
        .unwrap();
    nav_file.write_all("#[derive(Debug)]\n".as_bytes()).unwrap();
    nav_file
        .write_all("pub struct NavOrbit {\n   pub constellation: &'static str,\n   pub revisions: Vec<NavRevision>,\n}\n\n".as_bytes())
        .unwrap();
    nav_file.write_all("#[derive(Debug)]\n".as_bytes()).unwrap();
    nav_file
        .write_all("pub struct NavRevision {\n   pub major: &'static str,\n   pub minor: &'static str,\n   pub items: Vec<(&'static str,&'static str)>,\n}\n\n".as_bytes())
        .unwrap();
    nav_file.write_all("lazy_static! {\n".as_bytes()).unwrap();
    nav_file
        .write_all("   pub static ref NAV_ORBITS: Vec<NavOrbit> = vec![\n".as_bytes())
        .unwrap();
    for constellation in constellations {
        let c = constellation["constellation"].as_str().unwrap();
        nav_file.write_all("      NavOrbit {\n".as_bytes()).unwrap();
        nav_file
            .write_all(format!("         constellation: \"{}\",\n", c).as_bytes())
            .unwrap();
        nav_file
            .write_all("         revisions: vec![\n".as_bytes())
            .unwrap();
        for rev in constellation["revisions"].as_array().unwrap() {
            nav_file
                .write_all("            NavRevision {\n".as_bytes())
                .unwrap();
            let major = rev["revision"]["major"].as_u64().unwrap() as u8;
            let minor = rev["revision"]["minor"].as_u64().unwrap_or(0) as u8;
            nav_file
                .write_all(format!("               major: \"{}\",\n", major).as_bytes())
                .unwrap();
            nav_file
                .write_all(format!("               minor: \"{}\",\n", minor).as_bytes())
                .unwrap();
            nav_file
                .write_all("               items: vec![\n".as_bytes())
                .unwrap();
            let content = rev["content"].as_object().unwrap();
            for k in content.keys() {
                nav_file
                    .write_all(
                        format!("                  (\"{}\",{}),\n", k, content[k]).as_bytes(),
                    )
                    .unwrap();
            }
            nav_file.write_all("            ]},\n".as_bytes()).unwrap();
        }
        nav_file.write_all("         ],\n".as_bytes()).unwrap();
        nav_file.write_all("      },\n".as_bytes()).unwrap();
    }
    nav_file.write_all("   ];\n".as_bytes()).unwrap();
    nav_file.write_all("}".as_bytes()).unwrap();
}

fn main() {
    build_nav_database();
}
