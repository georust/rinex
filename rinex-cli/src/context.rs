use crate::fops::filename;
use crate::Cli;
use log::info;
use rinex::prelude::*;

#[derive(Debug, Clone)]
pub struct Context {
    pub prefix: String,
    pub primary_rinex: Rinex,
    pub to_merge: Option<Rinex>,
    pub nav_rinex: Option<Rinex>,
    pub ground_position: Option<(f64, f64, f64)>,
}

impl Context {
    fn create_workdir(fp: &str) {
        std::fs::create_dir_all(fp).expect(&format!("failed to create workdir \"{}\"", fp));
    }
    pub fn new(cli: &Cli) -> Self {
        let fp = cli.input_path();
        let prefix = env!("CARGO_MANIFEST_DIR").to_owned() + "/product/" + &filename(fp);
        Self::create_workdir(&prefix);

        let primary_rinex = Rinex::from_file(fp).expect("failed to parse primary rinex");

        let nav_rinex = cli.nav_context();

        Self {
            prefix: prefix.clone(),
            to_merge: cli.to_merge(),
            ground_position: {
                if let Some(ref pos) = primary_rinex.header.coords {
                    info!("ground position {:?} (ECEF)", pos);
                    Some(*pos)
                } else if let Some(ref nav) = nav_rinex {
                    if let Some(pos) = nav.header.coords {
                        info!("ground position {:?} (ECEF)", pos);
                        Some(pos)
                    } else {
                        if let Some(pos) = cli.manual_position() {
                            info!("manual ground position {:?} (ECEF)", pos);
                            Some(pos)
                        } else {
                            trace!("ground position undetermined");
                            None
                        }
                    }
                } else {
                    if let Some(pos) = cli.manual_position() {
                        info!("manual ground position {:?} (ECEF)", pos);
                        Some(pos)
                    } else {
                        trace!("ground position undetermined");
                        None
                    }
                }
            },
            primary_rinex,
            nav_rinex,
        }
    }
}
