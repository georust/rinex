//! File Quality opmode
use log::{error, info, warn};
use tl::{parse as html_parser, ParserOptions as HtmlParserOptions};

use std::{
    fs::{read_to_string, File},
    io::Write,
};

use crate::cli::{Cli, Context};

use rinex_qc::prelude::{QcConfig, QcExtraPage, QcReport, Render};

/// Quality check report
pub enum Report {
    /// New report generation/synthesis
    Pending(QcReport),
    /// Report iteration (preserved past run)
    Iteration(String),
}

impl Report {
    /// Create a new report
    pub fn new(cli: &Cli, ctx: &Context) -> Self {
        let cfg = QcConfig::default();
        info!("using default QC configuration: {:#?}", cfg);

        let report_path = ctx.workspace.root.join("index.html");
        let hash_path = ctx.workspace.root.join(".hash");
        if !cli.force_report_synthesis() && report_path.exists() && hash_path.exists() {
            // determine whether we can preserve previous report or not
            if let Ok(content) = read_to_string(hash_path) {
                if let Ok(prev_hash) = content.parse::<u64>() {
                    if prev_hash == cli.hash() {
                        if let Ok(content) = read_to_string(report_path) {
                            // parse prev HTML
                            let opts = HtmlParserOptions::new();
                            match html_parser(&content, opts) {
                                Ok(parsed) => {
                                    info!("preserving previous report");
                                    Self::Iteration(content.clone())
                                },
                                Err(e) => {
                                    error!("illegal html content: {}", e);
                                    warn!("forcing new report synthesis");
                                    Self::Pending(QcReport::new(&ctx.data, cfg))
                                },
                            }
                        } else {
                            error!("failed to parse previous report");
                            warn!("forcing new report synthesis");
                            Self::Pending(QcReport::new(&ctx.data, cfg))
                        }
                    } else {
                        info!("generating new report");
                        Self::Pending(QcReport::new(&ctx.data, cfg))
                    }
                } else {
                    error!("failed to parse hashed value");
                    warn!("forcing new report synthesis");
                    Self::Pending(QcReport::new(&ctx.data, cfg))
                }
            } else {
                // new report
                info!("generating new report");
                Self::Pending(QcReport::new(&ctx.data, cfg))
            }
        } else {
            // new report
            info!("generating new report");
            Self::Pending(QcReport::new(&ctx.data, cfg))
        }
    }
    /// Customize report with extra page
    pub fn customize(&mut self, id: &str, page: QcExtraPage) {
        match self {
            Self::Pending(report) => report.add_chapter(page),
            Self::Iteration(content) => {
                let opts = HtmlParserOptions::new().track_ids().track_classes();
                let mut html = html_parser(&content, opts).unwrap_or_else(|e| {
                    panic!("html customization failed unexpectedly: {}", e);
                });
                if let Some(node) = html.get_element_by_id("extra") {
                    println!("{:?}", node);
                }
            },
        }
    }
    /// Render as html
    fn render(&self) -> String {
        match self {
            Self::Pending(report) => report.render().into_string(),
            Self::Iteration(report) => String::default(),
        }
    }
    /// Generate (dump) report
    pub fn generate(&self, cli: &Cli, ctx: &Context) -> std::io::Result<()> {
        let html = self.render();
        let path = ctx.workspace.root.join("index.html");

        let mut fd = File::create(&path)?;
        write!(fd, "{}", html)?;
        info!("{} report generated", path.display());

        // store past settings
        if let Ok(mut fd) = File::create(ctx.workspace.root.join(".hash")) {
            let _ = write!(fd, "{}", cli.hash());
        }

        Ok(())
    }
}
