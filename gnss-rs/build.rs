use std::env;
use std::io::Write;
use std::path::Path;

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
    let db_content = std::fs::read_to_string("data/sbas.json").unwrap();

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
    build_sbas_helper();
}
