use std::env;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR")
        .unwrap();
    let nav_path = Path::new(&out_dir).join("nav_data.rs");
    let mut nav_file = std::fs::File::create(&nav_path).unwrap();
    let nav_data = std::fs::read_to_string("navigation.json")
        .unwrap();
    let json : serde_json::Value = serde_json::from_str(&nav_data)
        .unwrap();
    let constellations = json.as_array()
        .unwrap();
    nav_file
        .write_all("use lazy_static::lazy_static;\n\n".as_bytes())
        .unwrap();
    nav_file
        .write_all("#[derive(Debug)]\n".as_bytes())
        .unwrap();
    nav_file
        .write_all("struct NavMessage {\n   pub constellation: &'static str,\n   pub revisions: Vec<NavRevision>,\n}\n\n".as_bytes())
        .unwrap();
    nav_file
        .write_all("#[derive(Debug)]\n".as_bytes())
        .unwrap();
    nav_file
        .write_all("struct NavRevision {\n   major: &'static str,\n   minor: &'static str,\n   items: Vec<(&'static str,&'static str)>,\n}\n\n".as_bytes())
        .unwrap();
    nav_file
        .write_all("lazy_static! {\n".as_bytes())
        .unwrap();
    nav_file
        .write_all("   static ref NAV_MESSAGES: Vec<NavMessage> = vec![\n".as_bytes())
        .unwrap();
    for constellation in constellations {
        let c = constellation["constellation"].as_str()
            .unwrap();
        nav_file
            .write_all("      NavMessage {\n".as_bytes())
            .unwrap();
        nav_file
            .write_all(format!("         constellation: \"{}\",\n", c).as_bytes())
            .unwrap();
        nav_file
            .write_all("         revisions: vec![\n".as_bytes())
            .unwrap();
        for rev in constellation["revisions"].as_array()
            .unwrap() {
            nav_file
                .write_all("            NavRevision {\n".as_bytes())
                .unwrap();
            let major = rev["revision"]["major"].as_u64()
                .unwrap() as u8;
            let minor = rev["revision"]["minor"].as_u64()
                .unwrap_or(0) as u8;
            nav_file
                .write_all(format!("               major: \"{}\",\n", major).as_bytes())
                .unwrap();
            nav_file
                .write_all(format!("               minor: \"{}\",\n", minor).as_bytes())
                .unwrap();
            nav_file
                .write_all("               items: vec![\n".as_bytes())
                .unwrap();
            let content = rev["content"].as_object()
                .unwrap();
            for k in content.keys() {
                nav_file
                    .write_all(format!("                  (\"{}\",{}),\n", k, content[k]).as_bytes())
                    .unwrap();
            }
            nav_file
                .write_all("            ]},\n".as_bytes())
                .unwrap();
        }
        nav_file
            .write_all("         ],\n".as_bytes())
            .unwrap();
        nav_file
            .write_all("      },\n".as_bytes())
            .unwrap();
    }
    nav_file
        .write_all("   ];\n".as_bytes())
        .unwrap();
    nav_file
        .write_all("}".as_bytes())
        .unwrap();
}
