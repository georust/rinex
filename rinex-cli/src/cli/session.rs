use log::{info, warn};
use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::cli::tools::open_with_web_browser;
use rinex::prelude::GroundPosition;
use rinex_qc::prelude::DataContext;

use horrorshow::{box_html, helper::doctype, html, RenderBox};
use walkdir::WalkDir;

/// CLI Session
pub struct Session {
    /// Quiet option
    pub quiet: bool,
    /// Workspace, either manually or automatically defined
    pub workspace: PathBuf,
    /// Data context defined by user
    pub data: DataContext,
    /// Possible position of a geodetic marker contained in self.
    /// This is currently mandatory in -p opmode since it only supports
    /// comparison to a geodetic marker. Will have to be improved
    /// once we support dynamic positioning or if we ever support RTK.
    /// Geodetic marker position is either defined (by priority)
    ///  1. manually (command args)
    ///  2. automatically (picked up in [DataContext])
    pub geodetic_marker: Option<GroundPosition>,
}

/// Workspace snapshot is built in runtime
/// to represent the state of the workspace in the front page
#[derive(Default)]
struct WorkspaceSnapshot {
    /// Per Year (4 digits), per DOY (3 digits)
    pub products: HashMap<u16, HashMap<u16, Vec<String>>>,
}

impl WorkspaceSnapshot {
    fn new(workspace: &str) -> Self {
        // /Year#1
        //   /DOY#1
        //      /product1
        //      /product2
        //   /DOY#2
        //      /product1
        //      /product2
        // /Year#2
        //   /DOY#1
        //      /product1
        //      /product2
        //   /DOY#2
        //      /product1
        //      /product2
        let mut snap = Self::default();
        let mut doy = 0_u16;
        let mut year = 0_u16;
        let mut corrupted_doy = false;
        let mut corrupted_year = false;
        let walkdir = WalkDir::new(workspace).min_depth(1).max_depth(3);
        for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                let filename = match entry.path().file_name() {
                    Some(name) => name.to_string_lossy().to_string(),
                    _ => continue,
                };
                match entry.depth() {
                    1 => {
                        // println!("[1] FILE NAME \"{}\"", filename.to_string_lossy());
                        if let Ok(y) = filename.parse::<u16>() {
                            year = y;
                            corrupted_year = false;
                        } else {
                            warn!("Corrupted workspace - expecting __YEAR__/DOY/FOLDER");
                            corrupted_year = true;
                        }
                    },
                    2 => {
                        // println!("[2] FILE NAME \"{}\"", filename.to_string_lossy());
                        if !corrupted_year {
                            if let Ok(day) = filename.parse::<u16>() {
                                doy = day;
                                corrupted_doy = false;
                            } else {
                                warn!("Corrupted workspace - expecting YEAR/__DOY__/INDEX");
                                corrupted_doy = true;
                            }
                        }
                    },
                    3 => {
                        // println!("[3] FILE NAME \"{}\"", filename.to_string_lossy());
                        if !corrupted_year && !corrupted_doy {
                            if let Some(doys) = snap.products.get_mut(&year) {
                                if let Some(products) = doys.get_mut(&doy) {
                                    products.push(filename.clone());
                                } else {
                                    doys.insert(doy, vec![filename.clone()]);
                                }
                            } else {
                                let mut inner = HashMap::<u16, Vec<String>>::new();
                                inner.insert(doy, vec![filename.clone()]);
                                snap.products.insert(year, inner);
                            }
                        }
                    },
                    _ => {},
                }
            }
        }
        snap
    }
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            h2(class="title") {
                : "Workspace"
            }
            table(class="table is-bordered; style=\"margin-bottom: 20px\"") {
                tr {
                    th {
                        : "YEAR"
                    }
                    th {
                        : "DOY"
                    }
                    th {
                        : "INDEX"
                    }
                }
                @ for (year, doys) in self.products.iter() {
                    @ for (doy, products) in doys {
                        @ for product in products {
                            tr {
                                td {
                                    a(href=format!("{:04}", year)) {
                                        : format!("{:04}", year)
                                    }
                                }
                                td {
                                    a(href=format!("{:04}/{:03}", year, doy)) {
                                        : format!("{:03}", doy)
                                    }
                                }
                                td {
                                    a(href=format!("{:04}/{:03}/{}", year, doy, product)) {
                                        : product
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Session {
    /*
     * Utility to create one directory within the workspace
     */
    pub fn create_subdir(&self, suffix: &str) {
        create_dir_all(self.workspace.join(suffix))
            .unwrap_or_else(|e| panic!("Failed to create directory {}: {:?}", suffix, e));
    }
    /*
     * Utility to create one file within the workspace
     */
    fn create_file(&self, suffix: &str) -> File {
        std::fs::File::create(self.workspace.join(suffix)).unwrap_or_else(|e| {
            panic!("Failed to create {}: {:?}", suffix, e);
        })
    }
    /*
     * Concludes Self by creating (overwriting) $WORKSPACE/index.html
     * which is the main entry point to browse all generated products.
     */
    pub fn finalize(&self) {
        let snapshot = WorkspaceSnapshot::new(&self.workspace.to_string_lossy());
        let html = Self::make_index_html(snapshot);
        let mut fd = self.create_file("index.html");
        write!(fd, "{}", html).unwrap_or_else(|e| {
            panic!("failed to generate $WORKSPACE/index.html - {:?}", e);
        });
        info!(
            "Concluding this session: open \"{}/index.html\"",
            self.workspace.display()
        );
        if !self.quiet {
            open_with_web_browser(&format!("{}/index.html", self.workspace.display()));
        }
    }
    fn georust_logo_url() -> &'static str {
        "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png"
    }
    fn wiki_url() -> &'static str {
        "https://github.com/georust/rinex/wiki"
    }
    fn github_url() -> &'static str {
        "https://github.com/georust/rinex"
    }
    /*
     * Main front page title
     */
    fn main_front_title() -> Box<dyn RenderBox + 'static> {
        box_html! {
            table {
                tr {
                    td {
                        : format!("RINEX-Cli v{}", env!("CARGO_PKG_VERSION"))
                    }
                }
            }
        }
    }
    /*
     * Creates index.html main page (entry point)
     */
    fn make_index_html(snapshot: WorkspaceSnapshot) -> String {
        format!(
            "{}",
            html! {
                : doctype::HTML;
                html {
                    head {
                         meta(charset="UTF-8");
                         meta(name="viewport", content="width=device-width, initial-scale=1");
                         link(rel="stylesheet", href="https:////cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css");
                         script(defer="true", src="https://use.fontawesome.com/releases/v5.3.1/js/all.js");
                         link(rel="icon", href=Self::georust_logo_url(), style="width:35px;height:35px;");
                         title: format!("RINEX-Cli v{}", env!("CARGO_PKG_VERSION"));
                    } // head
                    body {
                        div(id="body_title") {
                            img(src=Self::georust_logo_url(), style="width:100px;height:100px;") {}
                            h2(class="title") {
                                : Self::main_front_title()
                            }
                        }
                        div(id="wiki") {
                            table {
                                tr {
                                    br {
                                        a(href=Self::wiki_url()) {
                                            : "Online Documentation"
                                        }
                                    }
                                }
                                tr {
                                    a(href=Self::github_url()) {
                                        : "Source Code"
                                    }
                                }
                            }
                        }
                        div(id="workspace") {
                            : snapshot.to_inline_html()
                        }
                        div(id="help") {
                            h3(class="title") {
                                : "Facing problems ?"
                            }
                            h5(class="text") {
                                : "test"
                            }
                        }
                    }
                }
            }
        )
    }
}
