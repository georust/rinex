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
    pub atx_rinex: Option<Rinex>,
    pub ground_position: Option<GroundPosition>,
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
        let atx_rinex = cli.atx_context();

        let ground_position = match &primary_rinex.header.ground_position {
            Some(position) => {
				info!("identified position: {}", position);
				Some(position.clone())
            },
            _ => {
                if let Some(ref nav) = nav_rinex {
                    if let Some(pos) = &nav.header.ground_position {
						info!("identified position: {}", pos);
                        Some(pos.clone())
                    } else {
                        if let Some(pos) = &cli.manual_position() {
							info!("manual position: {}", pos);
                            Some(pos.clone())
                        } else {
                            trace!("undetermined ground position");
                            None
                        }
                    }
                } else {
                    if let Some(pos) = &cli.manual_position() {
						info!("manual position: {}", pos);
                        Some(pos.clone())
                    } else {
                        trace!("undetermined ground position");
                        None
                    }
                }
            },
        };

        Self {
            ground_position,
            prefix: prefix.clone(),
            to_merge: cli.to_merge(),
            primary_rinex,
            nav_rinex,
            atx_rinex,
        }
    }
}
