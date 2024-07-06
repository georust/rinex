//! File Quality opmode
use log::{debug, error, info, warn};
use tl::{parse as html_parser, ParserOptions as HtmlParserOptions, VDom};

use std::{
    fs::{read_to_string, File},
    io::Write,
};

use crate::cli::{Cli, Context};

use rinex_qc::prelude::{Markup, QcConfig, QcExtraPage, QcReport, Render};

/// Quality check report
pub enum Report<'a> {
    /// New report, first iteration
    Pending(QcReport),
    /// Reporting/analysis iteration.
    /// Report has previously been generated, we do not
    /// regenerate its entirety but only custom pages
    Iteration(VDom<'a>),
}

impl<'a> Report<'a> {
    /// Create a new report
    pub fn new(cli: &'a Cli, ctx: &'a Context) -> Self {
        let cfg = QcConfig::default();
        info!("using default QC configuration: {:#?}", cfg);

        let report_path = ctx.workspace.root.join("index.html");
        let hash_path = ctx.workspace.root.join(".hash");
        if report_path.exists() && hash_path.exists() {
            // determine whether we should preserve this report or not
            if let Ok(content) = read_to_string(hash_path) {
                if let Ok(prev_hash) = content.parse::<u64>() {
                    if prev_hash == cli.hash() {
                        if let Ok(html) = read_to_string(report_path) {
                            let opts = HtmlParserOptions::new().track_ids().track_classes();
                            //match html_parser(&html, opts) {
                            //    Ok(parsed) => {
                            //        info!("previous report is preserved");
                            //        Self::Iteration(parsed)
                            //    },
                            //    Err(e) => {
                            //        error!("illegal html content: {}", e);
                            //        warn!("forcing new report generation");
                            //        Self::Pending(QcReport::new(&ctx.data, cfg))
                            //    },
                            //}
                            Self::Pending(QcReport::new(&ctx.data, cfg))
                        } else {
                            error!("failed to parse previous report");
                            warn!("forcing new report generation");
                            Self::Pending(QcReport::new(&ctx.data, cfg))
                        }
                    } else {
                        info!("generating new report");
                        Self::Pending(QcReport::new(&ctx.data, cfg))
                    }
                } else {
                    error!("failed to parse hashed value");
                    warn!("forcing new report generation");
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
    pub fn customize(&mut self, page: QcExtraPage) {
        match self {
            Self::Pending(report) => report.add_chapter(page),
            Self::Iteration(report) => {}, //TODO
        }
    }
    /// Render as html
    fn to_html(&self) -> Markup {
        match self {
            Self::Pending(report) => report.render(),
            Self::Iteration(report) => Markup::default(), //TODO, report.render(),
        }
    }
    /// Generate (dump) report
    pub fn generate(&self, ctx: &Context) -> std::io::Result<()> {
        let html = self.to_html().into_string();
        let path = ctx.workspace.root.join("index.html");
        let mut fd = File::create(&path)?;
        write!(fd, "{}", html)?;
        info!("{} report generated", path.display());
        Ok(())
    }
}
